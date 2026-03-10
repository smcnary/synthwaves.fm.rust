require "rails_helper"

RSpec.describe MediaDownloadJob, type: :job do
  let(:user) { create(:user) }
  let(:track) { create(:track, :youtube) }
  let(:url) { "https://youtube.com/watch?v=#{track.youtube_video_id}" }
  let(:temp_mp3) { Tempfile.new(["test", ".mp3"]) }

  before do
    File.write(temp_mp3.path, "fake mp3 data")
  end

  after { temp_mp3.close! }

  describe "#perform" do
    it "downloads audio and attaches it to the track" do
      allow(MediaDownloadService).to receive(:download_audio).and_return(temp_mp3.path)
      allow(MetadataExtractor).to receive(:call).and_return({ duration: 200.0, bitrate: 192 })

      described_class.perform_now(track.id, url, user_id: user.id)
      track.reload

      expect(track.audio_file).to be_attached
      expect(track.download_status).to eq("completed")
      expect(track.duration).to eq(200.0)
      expect(track.bitrate).to eq(192)
      expect(track.file_format).to eq("mp3")
    end

    it "skips if audio file is already attached" do
      track.audio_file.attach(io: StringIO.new("existing"), filename: "existing.mp3", content_type: "audio/mpeg")

      expect(MediaDownloadService).not_to receive(:download_audio)
      described_class.perform_now(track.id, url, user_id: user.id)
    end

    it "sets status to downloading during execution" do
      statuses = []
      allow(MediaDownloadService).to receive(:download_audio) do
        statuses << track.reload.download_status
        temp_mp3.path
      end
      allow(MetadataExtractor).to receive(:call).and_return({})

      described_class.perform_now(track.id, url, user_id: user.id)
      expect(statuses).to include("downloading")
    end

    it "sets failed status on download error" do
      allow(MediaDownloadService).to receive(:download_audio)
        .and_raise(MediaDownloadService::Error, "yt-dlp failed: something went wrong")

      described_class.perform_now(track.id, url, user_id: user.id)
      track.reload

      expect(track.download_status).to eq("failed")
      expect(track.download_error).to include("something went wrong")
    end

    it "cleans up temp directory" do
      allow(MediaDownloadService).to receive(:download_audio).and_return(temp_mp3.path)
      allow(MetadataExtractor).to receive(:call).and_return({})

      described_class.perform_now(track.id, url, user_id: user.id)

      temp_dirs = Dir.glob(Rails.root.join("tmp/media_downloads/track_#{track.id}_*"))
      expect(temp_dirs).to be_empty
    end

    it "cleans up temp directory even on failure" do
      allow(MediaDownloadService).to receive(:download_audio)
        .and_raise(MediaDownloadService::Error, "fail")

      described_class.perform_now(track.id, url, user_id: user.id)

      temp_dirs = Dir.glob(Rails.root.join("tmp/media_downloads/track_#{track.id}_*"))
      expect(temp_dirs).to be_empty
    end

    it "broadcasts status updates" do
      allow(MediaDownloadService).to receive(:download_audio).and_return(temp_mp3.path)
      allow(MetadataExtractor).to receive(:call).and_return({})

      expect(Turbo::StreamsChannel).to receive(:broadcast_replace_to).at_least(:twice)

      described_class.perform_now(track.id, url, user_id: user.id)
    end
  end
end
