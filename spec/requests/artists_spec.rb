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

    it "displays album cover image as artist thumbnail" do
      artist = create(:artist, name: "Cover Artist")
      album = create(:album, artist: artist)
      album.cover_image.attach(
        io: StringIO.new("fake image data"),
        filename: "cover.jpg",
        content_type: "image/jpeg"
      )

      get artists_path

      expect(response.body).to include("Cover Artist")
      expect(response.body).not_to include("M10 9a3 3 0 100-6 3 3 0 000 6z")
    end

    it "shows fallback icon when artist has no album cover" do
      create(:artist, name: "No Cover Artist")

      get artists_path

      expect(response.body).to include("No Cover Artist")
      expect(response.body).to include("M10 9a3 3 0 100-6 3 3 0 000 6z")
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
