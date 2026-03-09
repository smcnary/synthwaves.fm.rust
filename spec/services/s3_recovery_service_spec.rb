require "rails_helper"

RSpec.describe S3RecoveryService, type: :service do
  let(:s3_client) { double("Aws::S3::Client") }
  let(:credentials) do
    {
      access_key_id: "test-key",
      secret_access_key: "test-secret",
      region: "us-east-1",
      bucket: "test-bucket",
      endpoint: "https://s3.example.com"
    }
  end
  let(:user) { create(:user) }

  before do
    allow(Rails.application.credentials).to receive(:linode).and_return(credentials)
    allow(Aws::S3::Client).to receive(:new).and_return(s3_client)
  end

  def s3_object(key:, size: 1024, last_modified: Time.current)
    instance_double(Aws::S3::Types::Object, key: key, size: size, last_modified: last_modified)
  end

  def head_response(content_type:, content_disposition: nil, etag: '"abc123"')
    instance_double(
      Aws::S3::Types::HeadObjectOutput,
      content_type: content_type,
      content_disposition: content_disposition,
      etag: etag
    )
  end

  def list_response(contents:, is_truncated: false, next_continuation_token: nil)
    instance_double(
      Aws::S3::Types::ListObjectsV2Output,
      contents: contents,
      is_truncated: is_truncated,
      next_continuation_token: next_continuation_token
    )
  end

  describe ".call" do
    context "with an empty bucket" do
      before do
        allow(s3_client).to receive(:list_objects_v2).and_return(
          list_response(contents: [])
        )
      end

      it "returns zero stats" do
        stats = described_class.call(user_email: user.email_address)

        expect(stats[:scanned]).to eq(0)
        expect(stats[:audio_created]).to eq(0)
        expect(stats[:video_created]).to eq(0)
      end
    end

    context "skipping backup files" do
      before do
        allow(s3_client).to receive(:list_objects_v2).and_return(
          list_response(contents: [
            s3_object(key: "backups/db/production-2026-01-01.sqlite3.gz")
          ])
        )
      end

      it "skips objects under backups/ prefix" do
        stats = described_class.call(user_email: user.email_address)

        expect(stats[:scanned]).to eq(1)
        expect(stats[:audio_created]).to eq(0)
        expect(s3_client).not_to have_received(:head_object) if s3_client.respond_to?(:head_object)
      end
    end

    context "skipping existing blobs" do
      before do
        ActiveStorage::Blob.create!(
          key: "existing-key",
          filename: "existing.mp3",
          content_type: "audio/mpeg",
          byte_size: 1024,
          checksum: Base64.strict_encode64(Digest::MD5.digest("test")),
          service_name: "test"
        )

        allow(s3_client).to receive(:list_objects_v2).and_return(
          list_response(contents: [
            s3_object(key: "existing-key")
          ])
        )
      end

      it "skips objects that already have a blob record" do
        stats = described_class.call(user_email: user.email_address)

        expect(stats[:existing]).to eq(1)
      end
    end

    context "recovering audio files in dry-run mode" do
      before do
        allow(s3_client).to receive(:list_objects_v2).and_return(
          list_response(contents: [s3_object(key: "abc123", size: 5_000_000)])
        )
        allow(s3_client).to receive(:head_object).and_return(
          head_response(content_type: "audio/mpeg", content_disposition: 'inline; filename="song.mp3"')
        )
        allow(s3_client).to receive(:get_object) do |args|
          FileUtils.cp(Rails.root.join("spec/fixtures/files/test.mp3"), args[:response_target])
        end
      end

      it "reports what would be created without writing" do
        stats = described_class.call(user_email: user.email_address, commit: false)

        expect(stats[:audio_created]).to eq(1)
        expect(Track.count).to eq(0)
        expect(Artist.count).to eq(0)
      end
    end

    context "recovering audio files in commit mode" do
      before do
        allow(s3_client).to receive(:list_objects_v2).and_return(
          list_response(contents: [s3_object(key: "abc123", size: 5_000_000)])
        )
        allow(s3_client).to receive(:head_object).and_return(
          head_response(content_type: "audio/mpeg", content_disposition: 'inline; filename="song.mp3"')
        )
        allow(s3_client).to receive(:get_object) do |args|
          FileUtils.cp(Rails.root.join("spec/fixtures/files/test.mp3"), args[:response_target])
        end
      end

      it "creates artist, album, track, blob, and attachment records" do
        stats = described_class.call(user_email: user.email_address, commit: true)

        expect(stats[:audio_created]).to eq(1)
        expect(Track.count).to eq(1)

        track = Track.last
        expect(track.title).to be_present
        expect(track.artist).to be_present
        expect(track.album).to be_present
        expect(track.audio_file).to be_attached
        expect(track.audio_file.blob.key).to eq("abc123")
      end

      it "does not trigger AudioConversionJob" do
        expect {
          described_class.call(user_email: user.email_address, commit: true)
        }.not_to have_enqueued_job(AudioConversionJob)
      end

      it "skips duplicate tracks" do
        # First run creates the track
        described_class.call(user_email: user.email_address, commit: true)
        expect(Track.count).to eq(1)

        # Remove the blob so the second scan doesn't skip at blob-check stage
        blob = ActiveStorage::Blob.find_by(key: "abc123")
        ActiveStorage::Attachment.where(blob: blob).destroy_all
        blob&.destroy

        stats = described_class.call(user_email: user.email_address, commit: true)
        expect(stats[:skipped]).to eq(1)
        expect(Track.count).to eq(1)
      end
    end

    context "recovering video files in commit mode" do
      before do
        allow(s3_client).to receive(:list_objects_v2).and_return(
          list_response(contents: [s3_object(key: "vid456", size: 100_000_000)])
        )
        allow(s3_client).to receive(:head_object).and_return(
          head_response(
            content_type: "video/mp4",
            content_disposition: 'attachment; filename="S01E03 - Pilot.mp4"',
            etag: '"d41d8cd98f00b204e9800998ecf8427e"'
          )
        )
      end

      it "creates video, blob, and attachment records" do
        stats = described_class.call(user_email: user.email_address, commit: true)

        expect(stats[:video_created]).to eq(1)
        expect(Video.count).to eq(1)

        video = Video.last
        expect(video.title).to eq("Pilot")
        expect(video.season_number).to eq(1)
        expect(video.episode_number).to eq(3)
        expect(video.status).to eq("ready")
        expect(video.user).to eq(user)
        expect(video.file).to be_attached
        expect(video.file.blob.key).to eq("vid456")
      end

      it "does not trigger VideoConversionJob" do
        expect {
          described_class.call(user_email: user.email_address, commit: true)
        }.not_to have_enqueued_job(VideoConversionJob)
      end
    end

    context "handling pagination" do
      before do
        allow(s3_client).to receive(:list_objects_v2)
          .with(hash_including(prefix: ""))
          .and_return(list_response(
            contents: [s3_object(key: "backups/db/old.gz")],
            is_truncated: true,
            next_continuation_token: "token1"
          ))
        allow(s3_client).to receive(:list_objects_v2)
          .with(hash_including(continuation_token: "token1"))
          .and_return(list_response(
            contents: [s3_object(key: "backups/db/old2.gz")],
            is_truncated: false
          ))
      end

      it "follows continuation tokens" do
        stats = described_class.call(user_email: user.email_address)

        expect(stats[:scanned]).to eq(2)
        expect(s3_client).to have_received(:list_objects_v2).twice
      end
    end

    context "handling per-object errors" do
      before do
        allow(s3_client).to receive(:list_objects_v2).and_return(
          list_response(contents: [
            s3_object(key: "good-audio", size: 1024),
            s3_object(key: "bad-audio", size: 1024)
          ])
        )
        allow(s3_client).to receive(:head_object).with(hash_including(key: "good-audio")).and_return(
          head_response(content_type: "audio/mpeg", content_disposition: 'filename="good.mp3"')
        )
        allow(s3_client).to receive(:head_object).with(hash_including(key: "bad-audio")).and_return(
          head_response(content_type: "audio/mpeg", content_disposition: 'filename="bad.mp3"')
        )
        allow(s3_client).to receive(:get_object) do |args|
          if args[:key] == "good-audio"
            FileUtils.cp(Rails.root.join("spec/fixtures/files/test.mp3"), args[:response_target])
          else
            raise Aws::S3::Errors::ServiceError.new(nil, "download failed")
          end
        end
      end

      it "continues processing after individual errors" do
        stats = described_class.call(user_email: user.email_address, commit: true)

        expect(stats[:errors]).to eq(1)
        expect(stats[:audio_created]).to eq(1)
      end
    end

    context "with image files" do
      before do
        allow(s3_client).to receive(:list_objects_v2).and_return(
          list_response(contents: [s3_object(key: "img789")])
        )
        allow(s3_client).to receive(:head_object).and_return(
          head_response(content_type: "image/jpeg", content_disposition: 'filename="cover.jpg"')
        )
      end

      it "logs images for manual review and counts as skipped" do
        stats = described_class.call(user_email: user.email_address)

        expect(stats[:skipped]).to eq(1)
      end
    end
  end

  describe "filename parsing" do
    let(:service) { described_class.new(user_email: user.email_address) }

    it "parses quoted Content-Disposition filenames" do
      result = service.send(:parse_filename, 'inline; filename="my song.mp3"', "fallback-key")
      expect(result).to eq("my song.mp3")
    end

    it "parses unquoted Content-Disposition filenames" do
      result = service.send(:parse_filename, "attachment; filename=song.mp3", "fallback-key")
      expect(result).to eq("song.mp3")
    end

    it "parses URL-encoded filenames" do
      result = service.send(:parse_filename, "inline; filename=my%20song.mp3", "fallback-key")
      expect(result).to eq("my song.mp3")
    end

    it "falls back to S3 key basename when no Content-Disposition" do
      result = service.send(:parse_filename, nil, "uploads/abc123/song.mp3")
      expect(result).to eq("song.mp3")
    end
  end
end
