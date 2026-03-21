class YoutubeVideoImportService
  include ThumbnailAttachable

  class Error < StandardError; end

  SINGLES_ALBUM_TITLE = "YouTube Singles"

  def self.call(url, api_key:, user:, category: "music")
    new(url, category: category, api_key: api_key, user: user).call
  end

  def initialize(url, api_key:, user:, category: "music")
    @url = url
    @category = category
    @api_key = api_key
    @user = user
    @video_id = YoutubeUrlParser.extract_video_id(url)
    raise Error, "Invalid YouTube video URL" if @video_id.blank?
  end

  def call
    existing_track = @user.tracks.find_by(youtube_video_id: @video_id)
    return existing_track if existing_track

    details = fetch_video_details

    enriched = YoutubeMetadataEnricher.call(title: details[:title], channel_name: details[:channel_name])

    artist = @user.artists.find_or_create_by!(name: enriched[:artist] || "Unknown Artist") do |a|
      a.category = @category
    end

    album = @user.albums.find_or_create_by!(title: SINGLES_ALBUM_TITLE, artist: artist)

    if details[:thumbnail_url].present? && !album.cover_image.attached?
      attach_thumbnail(album, details[:thumbnail_url])
    end

    next_track_number = (album.tracks.maximum(:track_number) || 0) + 1

    @user.tracks.create!(
      title: enriched[:title],
      artist: artist,
      album: album,
      youtube_video_id: @video_id,
      duration: details[:duration],
      track_number: next_track_number
    )
  end

  private

  def fetch_video_details
    if @api_key.present?
      api = YoutubeAPIService.new(api_key: @api_key)
      details = api.fetch_video_details([@video_id]).first
      raise Error, "Video not found" if details.nil?
      details
    else
      MediaDownloadService.fetch_metadata("https://www.youtube.com/watch?v=#{@video_id}")
    end
  end
end
