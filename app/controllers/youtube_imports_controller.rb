class YoutubeImportsController < ApplicationController
  before_action :require_feature

  def new
  end

  def create
    url = params[:youtube_url]

    unless YoutubeUrlParser.playlist_url?(url)
      flash.now[:alert] = "Please enter a valid YouTube playlist URL."
      render :new, status: :unprocessable_content
      return
    end

    album = YoutubePlaylistImportService.call(url)
    redirect_to album_path(album), notice: "Playlist imported successfully! #{album.tracks.count} tracks added."
  rescue YoutubePlaylistImportService::Error, YoutubeAPIService::Error => e
    flash.now[:alert] = e.message
    render :new, status: :unprocessable_content
  end

  private

  def require_feature
    redirect_to root_path, alert: "This feature is not available." unless Flipper.enabled?(:youtube_import, Current.user)
  end
end
