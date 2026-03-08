class YoutubeUrlParser
  VIDEO_PATTERNS = [
    %r{youtube\.com/watch\?.*v=(?<id>[a-zA-Z0-9_-]{11})},
    %r{youtu\.be/(?<id>[a-zA-Z0-9_-]{11})},
    %r{youtube\.com/live/(?<id>[a-zA-Z0-9_-]{11})},
    %r{youtube\.com/embed/(?<id>[a-zA-Z0-9_-]{11})}
  ].freeze

  PLAYLIST_PATTERNS = [
    %r{[?&]list=(?<id>[a-zA-Z0-9_-]+)}
  ].freeze

  def self.extract_video_id(url)
    return nil if url.blank?

    VIDEO_PATTERNS.each do |pattern|
      match = url.match(pattern)
      return match[:id] if match
    end
    nil
  end

  def self.extract_playlist_id(url)
    return nil if url.blank?

    PLAYLIST_PATTERNS.each do |pattern|
      match = url.match(pattern)
      return match[:id] if match
    end
    nil
  end

  def self.video_url?(url)
    extract_video_id(url).present?
  end

  def self.playlist_url?(url)
    extract_playlist_id(url).present?
  end
end
