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

    category = params[:category].presence || "music"
    YoutubeImportJob.perform_later(url, category: category)
    redirect_to library_path, notice: "Playlist import started! It will appear in your library when ready."
  end

  private

  def require_feature
    redirect_to root_path, alert: "This feature is not available." unless Flipper.enabled?(:youtube_import, Current.user)
  end
end
