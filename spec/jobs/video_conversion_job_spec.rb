require "rails_helper"

RSpec.describe VideoConversionJob, type: :job do
  let(:user) { create(:user) }

  describe "#perform" do
    it "sets status to failed on error" do
      video = create(:video, user: user, status: "processing", file_format: "mkv")
      video.file.attach(io: StringIO.new("fake video"), filename: "test.mkv", content_type: "video/x-matroska")

      allow(VideoMetadataExtractor).to receive(:call).and_raise(StandardError, "ffprobe failed")

      described_class.new.perform(video.id)

      video.reload
      expect(video.status).to eq("failed")
      expect(video.error_message).to eq("ffprobe failed")
    end

    it "does nothing when no file is attached" do
      video = create(:video, user: user, status: "processing")

      expect { described_class.new.perform(video.id) }.not_to raise_error
      expect(video.reload.status).to eq("processing")
    end
  end

  describe "#needs_conversion?" do
    let(:job) { described_class.new }

    it "returns true for non-h264 video" do
      expect(job.send(:needs_conversion?, {video_codec: "vp9", audio_codec: "aac"})).to be true
    end

    it "returns true for non-aac audio" do
      expect(job.send(:needs_conversion?, {video_codec: "h264", audio_codec: "opus"})).to be true
    end

    it "returns false for h264+aac" do
      expect(job.send(:needs_conversion?, {video_codec: "h264", audio_codec: "aac"})).to be false
    end

    it "returns true when video_codec is nil" do
      expect(job.send(:needs_conversion?, {video_codec: nil})).to be true
    end
  end
end
