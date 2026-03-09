class VideoMetadataExtractor
  def self.call(file_path)
    new(file_path).call
  end

  def initialize(file_path)
    @file_path = file_path
  end

  def call
    output = `ffprobe -v quiet -print_format json -show_format -show_streams #{Shellwords.escape(@file_path)} 2>/dev/null`
    return {} if output.blank?

    data = JSON.parse(output)
    video_stream = data.dig("streams")&.find { |s| s["codec_type"] == "video" }
    audio_stream = data.dig("streams")&.find { |s| s["codec_type"] == "audio" }
    format = data["format"] || {}

    {
      duration: format["duration"]&.to_f,
      width: video_stream&.dig("width"),
      height: video_stream&.dig("height"),
      video_codec: video_stream&.dig("codec_name"),
      audio_codec: audio_stream&.dig("codec_name"),
      bitrate: format["bit_rate"] ? (format["bit_rate"].to_i / 1000) : nil,
      container: format["format_name"]
    }
  end
end
