require "rails_helper"

RSpec.describe VideoMetadataExtractor do
  describe ".call" do
    it "returns a hash of video metadata" do
      ffprobe_output = {
        streams: [
          {codec_type: "video", codec_name: "h264", width: 1920, height: 1080},
          {codec_type: "audio", codec_name: "aac"}
        ],
        format: {duration: "120.5", bit_rate: "5000000", format_name: "mov,mp4,m4a,3gp,3g2,mj2"}
      }.to_json

      allow_any_instance_of(VideoMetadataExtractor).to receive(:`).and_return(ffprobe_output)

      result = VideoMetadataExtractor.call("/fake/path.mp4")

      expect(result[:duration]).to eq(120.5)
      expect(result[:width]).to eq(1920)
      expect(result[:height]).to eq(1080)
      expect(result[:video_codec]).to eq("h264")
      expect(result[:audio_codec]).to eq("aac")
      expect(result[:bitrate]).to eq(5000)
      expect(result[:container]).to eq("mov,mp4,m4a,3gp,3g2,mj2")
    end

    it "returns empty hash when ffprobe output is blank" do
      allow_any_instance_of(VideoMetadataExtractor).to receive(:`).and_return("")

      result = VideoMetadataExtractor.call("/fake/path.mp4")

      expect(result).to eq({})
    end

    it "handles missing streams gracefully" do
      ffprobe_output = {streams: [], format: {duration: "60.0"}}.to_json
      allow_any_instance_of(VideoMetadataExtractor).to receive(:`).and_return(ffprobe_output)

      result = VideoMetadataExtractor.call("/fake/path.mp4")

      expect(result[:duration]).to eq(60.0)
      expect(result[:width]).to be_nil
      expect(result[:height]).to be_nil
      expect(result[:video_codec]).to be_nil
      expect(result[:audio_codec]).to be_nil
      expect(result[:container]).to be_nil
    end

    it "returns container from format_name for mkv files" do
      ffprobe_output = {
        streams: [
          {codec_type: "video", codec_name: "h264", width: 1280, height: 720},
          {codec_type: "audio", codec_name: "aac"}
        ],
        format: {duration: "90.0", bit_rate: "3000000", format_name: "matroska,webm"}
      }.to_json

      allow_any_instance_of(VideoMetadataExtractor).to receive(:`).and_return(ffprobe_output)

      result = VideoMetadataExtractor.call("/fake/path.mkv")

      expect(result[:container]).to eq("matroska,webm")
    end
  end
end
