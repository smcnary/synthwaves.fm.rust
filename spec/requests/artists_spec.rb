require "rails_helper"

RSpec.describe "Artists", type: :request do
  let(:user) { create(:user) }

  before { login_user(user) }

  describe "GET /artists" do
    it "returns success" do
      create(:artist)
      get artists_path
      expect(response).to have_http_status(:ok)
    end

    it "excludes podcast artists from index" do
      music_artist = create(:artist, name: "Music Band")
      podcast_artist = create(:artist, :podcast, name: "Podcast Show")

      get artists_path

      expect(response.body).to include("Music Band")
      expect(response.body).not_to include("Podcast Show")
    end
  end

  describe "GET /artists/:id" do
    it "returns success" do
      artist = create(:artist)
      get artist_path(artist)
      expect(response).to have_http_status(:ok)
    end
  end
end
