require "rails_helper"

RSpec.describe "Podcasts", type: :request do
  let(:user) { create(:user) }

  before { login_user(user) }

  describe "GET /podcasts" do
    it "returns success" do
      get podcasts_path
      expect(response).to have_http_status(:ok)
    end

    it "shows only podcast artists" do
      podcast_artist = create(:artist, :podcast, name: "My Podcast")
      music_artist = create(:artist, name: "My Band")

      get podcasts_path

      expect(response.body).to include("My Podcast")
      expect(response.body).not_to include("My Band")
    end

    it "shows empty state when no podcasts exist" do
      get podcasts_path
      expect(response.body).to include("No podcasts yet")
    end
  end

  describe "GET /podcasts/:id" do
    it "returns success for a podcast artist" do
      artist = create(:artist, :podcast)
      get podcast_path(artist)
      expect(response).to have_http_status(:ok)
    end

    it "returns not found for a music artist" do
      artist = create(:artist, category: "music")
      get podcast_path(artist)
      expect(response).to have_http_status(:not_found)
    end

    it "displays the podcast's albums" do
      artist = create(:artist, :podcast)
      album = create(:album, title: "Season 1", artist: artist)

      get podcast_path(artist)

      expect(response.body).to include("Season 1")
    end
  end
end
