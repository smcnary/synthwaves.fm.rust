class LyricsService
  LRCLIB_SEARCH_URL = "https://lrclib.net/api/search"
  USER_AGENT = "synthwaves.fm"
  TIMEOUT = 10

  def initialize(track)
    @track = track
  end

  def fetch
    return @track.lyrics if @track.lyrics.present?

    artist = @track.artist&.name.to_s
    title = @track.title.to_s

    lyrics = fetch_from_lrclib(artist, title)
    return nil unless lyrics

    @track.update!(lyrics: lyrics)
    lyrics
  end

  private

  def fetch_from_lrclib(artist, title)
    query = build_query(artist, title)
    return nil if query.blank?

    uri = URI(LRCLIB_SEARCH_URL)
    uri.query = URI.encode_www_form(q: query)

    response = Net::HTTP.start(uri.host, uri.port, use_ssl: true, open_timeout: TIMEOUT, read_timeout: TIMEOUT) do |http|
      req = Net::HTTP::Get.new(uri)
      req["User-Agent"] = USER_AGENT
      http.request(req)
    end

    return nil unless response.is_a?(Net::HTTPSuccess)

    results = JSON.parse(response.body)
    return nil if results.empty?

    # Prefer synced lyrics (keep timestamps for highlighting), fall back to plain
    synced = results.find { |r| r["syncedLyrics"].present? }
    if synced
      synced["syncedLyrics"]
    elsif results.first["plainLyrics"].present?
      results.first["plainLyrics"]
    end
  rescue
    nil
  end

  def build_query(artist, title)
    if title.include?(" - ")
      parts = title.split(" - ", 2)
      a = YoutubeMetadataEnricher.clean_for_search(parts[0])
      t = YoutubeMetadataEnricher.clean_for_search(parts[1])
      if a.present? && t.present?
        artist = a
        title = t
      end
    end

    "#{YoutubeMetadataEnricher.clean_for_search(artist)} #{YoutubeMetadataEnricher.clean_for_search(title)}".strip.presence
  end
end
