require "rails_helper"

RSpec.describe "Music", type: :request do
  describe "GET /music" do
    it "requires authentication" do
      get music_path
      expect(response).to redirect_to(new_session_path)
    end

    it "defaults to artists tab" do
      login_user(create(:user))
      get music_path
      expect(response).to have_http_status(:success)
      expect(response.body).to include("Music")
    end

    it "renders the artists tab" do
      login_user(create(:user))
      artist = create(:artist, name: "Test Artist", category: :music)

      get music_path(tab: "artists")

      expect(response).to have_http_status(:success)
      expect(response.body).to include("Test Artist")
    end

    it "renders the albums tab" do
      login_user(create(:user))
      artist = create(:artist, category: :music)
      album = create(:album, title: "Test Album", artist: artist)

      get music_path(tab: "albums")

      expect(response).to have_http_status(:success)
      expect(response.body).to include("Test Album")
    end

    it "renders the tracks tab" do
      login_user(create(:user))
      artist = create(:artist, category: :music)
      album = create(:album, artist: artist)
      track = create(:track, title: "Test Track", artist: artist, album: album)

      get music_path(tab: "tracks")

      expect(response).to have_http_status(:success)
      expect(response.body).to include("Test Track")
    end

    it "renders the podcasts tab" do
      login_user(create(:user))
      podcast_artist = create(:artist, :podcast, name: "My Great Podcast")

      get music_path(tab: "podcasts")

      expect(response).to have_http_status(:success)
      expect(response.body).to include("My Great Podcast")
    end

    it "does not show music artists in the podcasts tab" do
      login_user(create(:user))
      create(:artist, name: "Music Only Band", category: :music)

      get music_path(tab: "podcasts")

      expect(response).to have_http_status(:success)
      expect(response.body).not_to include("Music Only Band")
    end

    it "falls back to artists for invalid tab" do
      login_user(create(:user))
      get music_path(tab: "invalid")
      expect(response).to have_http_status(:success)
      expect(response.body).to include("Search artists...")
    end
  end
end
