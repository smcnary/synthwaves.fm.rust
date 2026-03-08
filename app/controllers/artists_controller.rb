class ArtistsController < ApplicationController
  def index
    @artists = Artist.music.includes(albums: { cover_image_attachment: :blob }).order(:name)
  end

  def show
    @artist = Artist.find(params[:id])
    @albums = @artist.albums.includes(:tracks).order(:year)
  end
end
