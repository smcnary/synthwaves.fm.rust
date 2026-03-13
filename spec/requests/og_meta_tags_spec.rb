require "rails_helper"

RSpec.describe "OG meta tags", type: :request do
  describe "GET / (landing page)" do
    it "returns success without authentication" do
      get root_path

      expect(response).to have_http_status(:ok)
    end

    it "includes OG title and description" do
      get root_path

      expect(response.body).to include('property="og:title"')
      expect(response.body).to include('property="og:description"')
      expect(response.body).to include('property="og:image"')
    end

    it "does not include noindex robots directive" do
      get root_path

      expect(response.body).not_to include("noindex")
    end
  end

  describe "authenticated pages remain behind auth" do
    let(:user) { create(:user) }
    let(:artist) { create(:artist, user: user) }
    let(:album) { create(:album, artist: artist) }
    let(:track) { create(:track, album: album, artist: artist) }
    let(:playlist) { create(:playlist, user: user) }

    it "redirects unauthenticated GET /albums/:id to login" do
      get album_path(album)
      expect(response).to redirect_to(new_session_path)
    end

    it "redirects unauthenticated GET /tracks/:id to login" do
      get track_path(track)
      expect(response).to redirect_to(new_session_path)
    end

    it "redirects unauthenticated GET /artists/:id to login" do
      get artist_path(artist)
      expect(response).to redirect_to(new_session_path)
    end

    it "redirects unauthenticated GET /playlists/:id to login" do
      get playlist_path(playlist)
      expect(response).to redirect_to(new_session_path)
    end
  end
end
