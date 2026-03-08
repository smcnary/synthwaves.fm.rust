require "rails_helper"

RSpec.describe "YoutubeImports", type: :request do
  let(:user) { create(:user) }

  before do
    login_user(user)
    Flipper.enable(:youtube_import)
    allow(Rails.application.credentials).to receive(:youtube_api_key).and_return("test_key")
  end

  describe "GET /youtube_imports/new" do
    it "returns success" do
      get new_youtube_import_path
      expect(response).to have_http_status(:ok)
    end
  end

  describe "POST /youtube_imports" do
    it "imports a YouTube playlist and redirects to the album" do
      stub_playlist_api_calls

      post youtube_imports_path, params: { youtube_url: "https://www.youtube.com/playlist?list=PLtest123" }

      album = Album.find_by(title: "Test Playlist")
      expect(album).to be_present
      expect(album.tracks.count).to eq(2)
      expect(response).to redirect_to(album_path(album))
    end

    it "rejects invalid URLs" do
      post youtube_imports_path, params: { youtube_url: "https://example.com" }
      expect(response).to have_http_status(:unprocessable_content)
    end

    it "handles API errors gracefully" do
      stub_request(:get, %r{googleapis\.com/youtube/v3/playlists})
        .to_return(
          status: 403,
          headers: { "Content-Type" => "application/json" },
          body: { error: { message: "API key invalid" } }.to_json
        )

      post youtube_imports_path, params: { youtube_url: "https://www.youtube.com/playlist?list=PLtest123" }
      expect(response).to have_http_status(:unprocessable_content)
    end
  end

  private

  def stub_playlist_api_calls
    stub_request(:get, "https://www.googleapis.com/youtube/v3/playlists")
      .with(query: hash_including(id: "PLtest123"))
      .to_return(
        status: 200,
        headers: { "Content-Type" => "application/json" },
        body: {
          items: [{
            snippet: {
              title: "Test Playlist",
              channelTitle: "Test Channel",
              thumbnails: { high: { url: "https://i.ytimg.com/vi/abc/hqdefault.jpg" } }
            },
            contentDetails: { itemCount: 2 }
          }]
        }.to_json
      )

    stub_request(:get, "https://www.googleapis.com/youtube/v3/playlistItems")
      .with(query: hash_including(playlistId: "PLtest123"))
      .to_return(
        status: 200,
        headers: { "Content-Type" => "application/json" },
        body: {
          items: [
            { snippet: { resourceId: { videoId: "vid1" }, title: "Song 1", position: 0, thumbnails: {} } },
            { snippet: { resourceId: { videoId: "vid2" }, title: "Song 2", position: 1, thumbnails: {} } }
          ]
        }.to_json
      )

    stub_request(:get, "https://www.googleapis.com/youtube/v3/videos")
      .with(query: hash_including(id: "vid1,vid2"))
      .to_return(
        status: 200,
        headers: { "Content-Type" => "application/json" },
        body: {
          items: [
            { id: "vid1", snippet: { title: "Song 1", channelTitle: "Test Channel" }, contentDetails: { duration: "PT3M45S" } },
            { id: "vid2", snippet: { title: "Song 2", channelTitle: "Test Channel" }, contentDetails: { duration: "PT4M10S" } }
          ]
        }.to_json
      )

    stub_request(:get, "https://i.ytimg.com/vi/abc/hqdefault.jpg")
      .to_return(status: 200, body: "fake_image_data", headers: { "Content-Type" => "image/jpeg" })
  end
end
