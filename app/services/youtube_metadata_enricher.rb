class YoutubeMetadataEnricher
  # Brackets: always remove
  BRACKET_NOISE = /\s*\[.*?\]/

  # Parenthesized noise: remove these, but preserve (feat. ...), (Remix), (Live ...), etc.
  PAREN_NOISE = /\s*\((?:official\s+(?:video|audio|music\s+video|lyric\s+video)|lyrics?|audio|video|music\s+video|hd|hq|4k|visuali[sz]er|clip\s+officiel|remastered(?:\s+\d{4})?)\s*\)/i

  # For aggressive search cleaning: strip ALL parens and brackets
  ALL_PARENS_AND_BRACKETS = /\s*(?:\[.*?\]|\(.*?\))/

  # Trailing noise words (without parens/brackets)
  TRAILING_NOISE = /\s*[-|]?\s*(?:official\s+(?:video|audio|music\s+video|lyric\s+video)|lyrics?|hd|hq|4k)\s*$/i

  def self.call(title:, channel_name:)
    new(title, channel_name).call
  end

  def self.clean_for_search(text)
    text.to_s.gsub(ALL_PARENS_AND_BRACKETS, "").gsub(TRAILING_NOISE, "").strip
  end

  def initialize(title, channel_name)
    @title = title.to_s.strip
    @channel_name = channel_name.to_s.strip
  end

  def call
    cleaned = clean_title(@title)

    if cleaned.include?(" - ")
      artist, title = cleaned.split(" - ", 2)
      artist = artist.strip
      title = title.strip

      if artist.present? && title.present?
        return {artist: artist, title: title, source: :parsed}
      end
    end

    {artist: @channel_name.presence || "Unknown Artist", title: cleaned.presence || @title, source: :channel}
  end

  private

  def clean_title(text)
    text
      .gsub(BRACKET_NOISE, "")
      .gsub(PAREN_NOISE, "")
      .gsub(TRAILING_NOISE, "")
      .strip
  end
end
