require "rails_helper"

RSpec.describe VideoMetadataExtractor do
  describe ".call" do
    it "returns a hash of video metadata" do
      ffprobe_output = {
        streams: [
          {codec_type: "video", codec_name: "h264", width: 1920, height: 1080},
          {codec_type: "audio", codec_name: "aac"}
        ],
        format: {duration: "120.5", bit_rate: "5000000"}
      }.to_json

      allow_any_instance_of(VideoMetadataExtractor).to receive(:`).and_return(ffprobe_output)

      result = VideoMetadataExtractor.call("/fake/path.mp4")

      expect(result[:duration]).to eq(120.5)
      expect(result[:width]).to eq(1920)
      expect(result[:height]).to eq(1080)
      expect(result[:video_codec]).to eq("h264")
      expect(result[:audio_codec]).to eq("aac")
      expect(result[:bitrate]).to eq(5000)
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
    end
  end
end
