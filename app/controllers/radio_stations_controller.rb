class RadioStationsController < ApplicationController
  before_action :require_feature

  def index
    @radio_stations = RadioStation.order(created_at: :desc)
  end

  def new
    @radio_station = RadioStation.new
  end

  def create
    @radio_station = Current.user.radio_stations.new(radio_station_params)

    video_id = YoutubeUrlParser.extract_video_id(@radio_station.youtube_url)
    if video_id.present?
      @radio_station.youtube_video_id = video_id
      fetch_oembed_metadata
    end

    if @radio_station.save
      redirect_to radio_stations_path, notice: "Radio station added."
    else
      render :new, status: :unprocessable_content
    end
  end

  def destroy
    radio_station = Current.user.radio_stations.find(params[:id])
    radio_station.destroy
    redirect_to radio_stations_path, notice: "Radio station removed."
  end

  private

  def require_feature
    redirect_to root_path, alert: "This feature is not available." unless Flipper.enabled?(:youtube_radio, Current.user)
  end

  def radio_station_params
    params.require(:radio_station).permit(:name, :youtube_url)
  end

  def fetch_oembed_metadata
    response = HTTP.get("https://www.youtube.com/oembed", params: {
      url: @radio_station.youtube_url,
      format: "json"
    })

    if response.status.success?
      data = response.parse
      @radio_station.name = data["title"] if @radio_station.name.blank?
      @radio_station.thumbnail_url = data["thumbnail_url"]
    end
  rescue HTTP::Error
    # oEmbed fetch failed — user can still provide name manually
  end
end
