class YoutubePlaylistImportService
  class Error < StandardError; end

  def self.call(url, category: "music")
    new(url, category: category).call
  end

  def initialize(url, category: "music")
    @url = url
    @category = category
    @playlist_id = YoutubeUrlParser.extract_playlist_id(url)
    raise Error, "Invalid YouTube playlist URL" if @playlist_id.blank?
  end

  def call
    api = YoutubeAPIService.new

    playlist_info = api.fetch_playlist_info(@playlist_id)
    playlist_items = api.fetch_playlist_items(@playlist_id)

    return nil if playlist_items.empty?

    video_ids = playlist_items.map { |item| item[:video_id] }
    video_details = api.fetch_video_details(video_ids)
    details_by_id = video_details.index_by { |v| v[:video_id] }

    artist = Artist.find_or_create_by!(name: playlist_info[:channel_name] || "Unknown Artist") do |a|
      a.category = @category
    end

    album = Album.find_or_create_by!(title: playlist_info[:title], artist: artist)

    if playlist_info[:thumbnail_url].present? && !album.cover_image.attached?
      attach_thumbnail(album, playlist_info[:thumbnail_url])
    end

    playlist_items.each do |item|
      details = details_by_id[item[:video_id]] || {}

      Track.find_or_create_by!(youtube_video_id: item[:video_id]) do |track|
        track.title = item[:title]
        track.artist = artist
        track.album = album
        track.track_number = item[:position] + 1
        track.duration = details[:duration]
      end
    end

    album
  end

  private

  def attach_thumbnail(album, thumbnail_url)
    response = HTTP.get(thumbnail_url)
    return unless response.status.success?

    content_type = response.content_type.mime_type
    extension = case content_type
    when "image/png" then "png"
    when "image/webp" then "webp"
    else "jpg"
    end

    album.cover_image.attach(
      io: StringIO.new(response.body.to_s),
      filename: "cover.#{extension}",
      content_type: content_type
    )
  rescue HTTP::Error
    # Thumbnail download failed — not critical, skip it
  end
end
