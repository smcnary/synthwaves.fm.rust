class LibraryController < ApplicationController
  def show
    @artist_count = Artist.music.count
    @album_count = Album.music.count
    @track_count = Track.music.count
    @total_duration = Track.music.sum(:duration)
    @recent_tracks = Track.music.includes(:artist, :album).order(created_at: :desc).limit(10)
  end
end
