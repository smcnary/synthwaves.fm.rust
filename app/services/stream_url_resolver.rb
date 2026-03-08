class StreamUrlResolver
  Result = Data.define(:stream_url, :name, :error)

  def self.call(url)
    new(url).resolve
  end

  def initialize(url)
    @url = url
  end

  def resolve
    response = HTTP.follow(max_hops: 5).get(@url)

    content_type = response.content_type.mime_type.to_s
    body = response.body.to_s

    if pls?(content_type, body)
      parse_pls(body)
    elsif m3u?(content_type, body)
      parse_m3u(body)
    else
      # Assume it's a direct stream URL
      Result.new(stream_url: @url, name: nil, error: nil)
    end
  rescue HTTP::Error, HTTP::TimeoutError => e
    Result.new(stream_url: nil, name: nil, error: "Could not fetch URL: #{e.message}")
  end

  private

  def pls?(content_type, body)
    content_type.include?("audio/x-scpls") ||
      @url.end_with?(".pls") ||
      body.strip.start_with?("[playlist]")
  end

  def m3u?(content_type, body)
    content_type.include?("audio/x-mpegurl") ||
      content_type.include?("audio/mpegurl") ||
      @url.end_with?(".m3u") ||
      @url.end_with?(".m3u8") ||
      body.strip.start_with?("#EXTM3U")
  end

  def parse_pls(body)
    stream_url = nil
    name = nil

    body.each_line do |line|
      line = line.strip
      if line =~ /\AFile\d+=(.+)\z/i
        stream_url ||= $1.strip
      elsif line =~ /\ATitle\d+=(.+)\z/i
        name ||= $1.strip
      end
    end

    if stream_url.present?
      Result.new(stream_url: resolve_redirects(stream_url), name: name.presence, error: nil)
    else
      Result.new(stream_url: nil, name: nil, error: "No stream URL found in PLS file")
    end
  end

  def parse_m3u(body)
    stream_url = nil
    name = nil

    body.each_line do |line|
      line = line.strip
      if line.start_with?("#EXTINF:")
        # Format: #EXTINF:duration,title
        name ||= line.sub(/\A#EXTINF:[^,]*,/, "").strip.presence
      elsif line.present? && !line.start_with?("#")
        stream_url ||= line
      end
    end

    if stream_url.present?
      Result.new(stream_url: resolve_redirects(stream_url), name: name.presence, error: nil)
    else
      Result.new(stream_url: nil, name: nil, error: "No stream URL found in M3U file")
    end
  end

  def resolve_redirects(url)
    response = HTTP.head(url)
    if response.status.redirect?
      response.headers["Location"] || url
    else
      url
    end
  rescue HTTP::Error
    url
  end
end
