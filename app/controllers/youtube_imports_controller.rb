class YoutubeImportsController < ApplicationController
  before_action :require_feature

  def new
  end

  def create
    url = params[:youtube_url]
    category = params[:category].presence || "music"

    if YoutubeUrlParser.playlist_url?(url)
      YoutubeImportJob.perform_later(url, category: category)
      redirect_to library_path, notice: "Playlist import started! It will appear in your library when ready."
    elsif YoutubeUrlParser.video_url?(url)
      track = YoutubeVideoImportService.call(url, category: category)
      redirect_to album_path(track.album), notice: "Video imported successfully!"
    else
      flash.now[:alert] = "Please enter a valid YouTube URL."
      render :new, status: :unprocessable_content
    end
  rescue YoutubeVideoImportService::Error => e
    flash.now[:alert] = e.message
    render :new, status: :unprocessable_content
  end

  private

  def require_feature
    redirect_to root_path, alert: "This feature is not available." unless Flipper.enabled?(:youtube_import, Current.user)
  end
end
