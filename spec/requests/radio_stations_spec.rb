require "rails_helper"

RSpec.describe "RadioStations", type: :request do
  let(:user) { create(:user) }

  before do
    login_user(user)
    Flipper.enable(:youtube_radio)
  end

  describe "GET /radio_stations" do
    it "returns success" do
      get radio_stations_path
      expect(response).to have_http_status(:ok)
    end

    it "displays radio stations" do
      station = create(:radio_station, user: user, name: "Lo-Fi Beats")
      get radio_stations_path
      expect(response.body).to include("Lo-Fi Beats")
    end
  end

  describe "GET /radio_stations/new" do
    it "returns success" do
      get new_radio_station_path
      expect(response).to have_http_status(:ok)
    end
  end

  describe "POST /radio_stations" do
    it "creates a radio station with valid YouTube URL" do
      stub_request(:get, %r{youtube\.com/oembed})
        .to_return(
          status: 200,
          headers: { "Content-Type" => "application/json" },
          body: { title: "Lo-Fi Beats", thumbnail_url: "https://i.ytimg.com/vi/jfKfPfyJRdk/hqdefault.jpg" }.to_json
        )

      expect {
        post radio_stations_path, params: { radio_station: {
          youtube_url: "https://www.youtube.com/watch?v=jfKfPfyJRdk"
        } }
      }.to change(RadioStation, :count).by(1)

      station = RadioStation.last
      expect(station.youtube_video_id).to eq("jfKfPfyJRdk")
      expect(station.name).to eq("Lo-Fi Beats")
      expect(response).to redirect_to(radio_stations_path)
    end

    it "creates a radio station with manual name and still fetches thumbnail" do
      stub_request(:get, %r{youtube\.com/oembed})
        .to_return(
          status: 200,
          headers: { "Content-Type" => "application/json" },
          body: { title: "Lo-Fi Beats", thumbnail_url: "https://i.ytimg.com/vi/jfKfPfyJRdk/hqdefault.jpg" }.to_json
        )

      expect {
        post radio_stations_path, params: { radio_station: {
          youtube_url: "https://www.youtube.com/watch?v=jfKfPfyJRdk",
          name: "My Radio"
        } }
      }.to change(RadioStation, :count).by(1)

      station = RadioStation.last
      expect(station.name).to eq("My Radio")
      expect(station.thumbnail_url).to eq("https://i.ytimg.com/vi/jfKfPfyJRdk/hqdefault.jpg")
    end

    it "rejects invalid URL" do
      post radio_stations_path, params: { radio_station: {
        youtube_url: "https://example.com/not-youtube",
        name: "Bad Station"
      } }
      expect(response).to have_http_status(:unprocessable_content)
    end
  end

  describe "DELETE /radio_stations/:id" do
    it "deletes the user's own station" do
      station = create(:radio_station, user: user)

      expect {
        delete radio_station_path(station)
      }.to change(RadioStation, :count).by(-1)

      expect(response).to redirect_to(radio_stations_path)
    end

    it "does not delete another user's station" do
      other_user = create(:user)
      station = create(:radio_station, user: other_user)

      expect {
        delete radio_station_path(station)
      }.not_to change(RadioStation, :count)
    end
  end
end
