require "rails_helper"

RSpec.describe "Albums", type: :request do
  let(:user) { create(:user) }

  before { login_user(user) }

  describe "GET /albums" do
    it "returns success" do
      create(:album)
      get albums_path
      expect(response).to have_http_status(:ok)
    end
  end

  describe "GET /albums/:id" do
    it "returns success" do
      album = create(:album)
      get album_path(album)
      expect(response).to have_http_status(:ok)
    end
  end

  describe "POST /albums/:id/create_playlist" do
    it "creates a playlist named after the album with all tracks in disc/track order" do
      album = create(:album, title: "Great Album")
      track3 = create(:track, album: album, disc_number: 2, track_number: 1, title: "Disc 2 Track 1")
      track1 = create(:track, album: album, disc_number: 1, track_number: 1, title: "Disc 1 Track 1")
      track2 = create(:track, album: album, disc_number: 1, track_number: 2, title: "Disc 1 Track 2")

      expect {
        post create_playlist_album_path(album)
      }.to change(Playlist, :count).by(1)

      playlist = user.playlists.last
      expect(playlist.name).to eq("Great Album")
      expect(playlist.playlist_tracks.order(:position).map(&:track)).to eq([track1, track2, track3])
      expect(response).to redirect_to(playlist_path(playlist))
    end

    it "requires authentication" do
      album = create(:album)
      reset!
      post create_playlist_album_path(album)
      expect(response).to redirect_to(new_session_path)
    end
  end
end
