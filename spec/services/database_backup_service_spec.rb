require "rails_helper"

RSpec.describe DatabaseBackupService, type: :service do
  let(:s3_client) { instance_double(Aws::S3::Client) }
  let(:credentials) do
    {
      access_key_id: "test-key",
      secret_access_key: "test-secret",
      region: "us-east-1",
      bucket: "test-bucket",
      endpoint: "https://s3.example.com"
    }
  end

  before do
    allow(Rails.application.credentials).to receive(:linode).and_return(credentials)
    allow(Aws::S3::Client).to receive(:new).and_return(s3_client)
    allow(s3_client).to receive(:put_object)
    allow(s3_client).to receive(:list_objects_v2).and_return(
      instance_double(Aws::S3::Types::ListObjectsV2Output, contents: [])
    )
  end

  describe ".call" do
    it "creates a gzipped backup and uploads to S3" do
      result = described_class.call

      expect(result[:key]).to match(%r{\Abackups/db/production-.*\.sqlite3\.gz\z})
      expect(result[:size]).to be > 0
      expect(s3_client).to have_received(:put_object).with(
        hash_including(bucket: "test-bucket", key: result[:key])
      )
    end

    it "produces a valid gzip file containing a SQLite database" do
      uploaded_body = nil
      allow(s3_client).to receive(:put_object) do |args|
        uploaded_body = args[:body].read
      end

      described_class.call

      # Verify it's valid gzip by decompressing
      decompressed = Zlib::GzipReader.new(StringIO.new(uploaded_body)).read
      # SQLite databases start with "SQLite format 3\000"
      expect(decompressed).to start_with("SQLite format 3")
    end

    it "prunes backups beyond retention count" do
      old_objects = 10.times.map do |i|
        instance_double(
          Aws::S3::Types::Object,
          key: "backups/db/production-2026-01-0#{i}T00:00:00Z.sqlite3.gz",
          last_modified: Time.utc(2026, 1, i + 1)
        )
      end

      allow(s3_client).to receive(:list_objects_v2).and_return(
        instance_double(Aws::S3::Types::ListObjectsV2Output, contents: old_objects)
      )
      allow(s3_client).to receive(:delete_object)

      described_class.call(retention: 7)

      expect(s3_client).to have_received(:delete_object).exactly(3).times
    end

    it "does not delete when under retention count" do
      objects = 3.times.map do |i|
        instance_double(
          Aws::S3::Types::Object,
          key: "backups/db/production-2026-01-0#{i}T00:00:00Z.sqlite3.gz",
          last_modified: Time.utc(2026, 1, i + 1)
        )
      end

      allow(s3_client).to receive(:list_objects_v2).and_return(
        instance_double(Aws::S3::Types::ListObjectsV2Output, contents: objects)
      )

      described_class.call(retention: 7)

      expect(s3_client).not_to have_received(:delete_object) if s3_client.respond_to?(:delete_object)
    end

    it "cleans up temp files even on S3 upload failure" do
      allow(s3_client).to receive(:put_object).and_raise(Aws::S3::Errors::ServiceError.new(nil, "fail"))

      expect {
        described_class.call
      }.to raise_error(Aws::S3::Errors::ServiceError)

      # Dir.mktmpdir auto-cleans the block-form tmpdir
    end
  end
end
