class AlbumsController < ApplicationController
  def index
    @albums = Album.music.includes(:artist).order(:title)
  end

  def show
    @album = Album.includes(:artist, tracks: :artist).find(params[:id])
    @sort = sort_column(Track, default: "disc_number")
    @direction = sort_direction

    scope = if @sort == "disc_number"
      @album.tracks.order(disc_number: @direction, track_number: @direction)
    else
      @album.tracks.order(@sort => @direction)
    end

    @total_tracks = scope.count
    @all_tracks = @album.tracks
    @pagy, @tracks = pagy(:offset, scope)
  end

  def create_playlist
    album = Album.find(params[:id])
    tracks = album.tracks.order(:disc_number, :track_number)
    playlist = Current.user.playlists.create!(name: album.title)

    tracks.each_with_index do |track, index|
      playlist.playlist_tracks.create!(track: track, position: index + 1)
    end

    redirect_to playlist, notice: "Playlist created from #{album.title}"
  end
end
