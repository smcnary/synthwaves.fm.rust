class MusicController < ApplicationController
  include Orderable

  TABS = %w[artists albums tracks podcasts].freeze

  def show
    @tab = params[:tab].presence_in(TABS) || "artists"

    case @tab
    when "artists"
      load_artists
    when "albums"
      load_albums
    when "tracks"
      load_tracks
    when "podcasts"
      load_podcasts
    end
  end

  private

  def load_artists
    @query = params[:q]
    @sort = sort_column(Artist, default: "name")
    @direction = sort_direction
    scope = Artist.music.includes(albums: {cover_image_attachment: :blob})
      .search(@query)
      .order(@sort => @direction)
    @pagy, @artists = pagy(:offset, scope, limit: 60)
  end

  def load_albums
    @query = params[:q]
    @sort = sort_column(Album, default: "title")
    @direction = sort_direction
    scope = Album.music.includes(:artist, cover_image_attachment: :blob)
      .search(@query)
      .order(@sort => @direction)
    @pagy, @albums = pagy(:offset, scope)
  end

  def load_tracks
    @query = params[:q]
    scope = Track.music.includes(:artist, :album).search(@query).order(:title)
    @pagy, @tracks = pagy(:offset, scope)
    @favorited_track_ids = Current.user.favorites.where(favorable_type: "Track").pluck(:favorable_id).to_set
  end

  def load_podcasts
    @query = params[:q]
    scope = Artist.podcast.includes(albums: {cover_image_attachment: :blob})
      .search(@query)
      .order(:name)
    @pagy, @podcasts = pagy(:offset, scope)
  end
end
