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

    it "sorts by disc/track number by default" do
      album = create(:album)
      track_b = create(:track, album: album, disc_number: 1, track_number: 2, title: "Beta")
      track_a = create(:track, album: album, disc_number: 1, track_number: 1, title: "Alpha")
      track_c = create(:track, album: album, disc_number: 2, track_number: 1, title: "Charlie")

      get album_path(album)

      expect(response.body.index(track_a.title)).to be < response.body.index(track_b.title)
      expect(response.body.index(track_b.title)).to be < response.body.index(track_c.title)
    end

    it "sorts by created_at desc (newest first)" do
      album = create(:album)
      old_track = create(:track, album: album, title: "Old Track", created_at: 2.days.ago)
      new_track = create(:track, album: album, title: "New Track", created_at: 1.hour.ago)

      get album_path(album, sort: "created_at", direction: "desc")

      expect(response.body.index(new_track.title)).to be < response.body.index(old_track.title)
    end

    it "sorts by title asc (alphabetical)" do
      album = create(:album)
      track_z = create(:track, album: album, title: "Zebra")
      track_a = create(:track, album: album, title: "Apple")

      get album_path(album, sort: "title", direction: "asc")

      expect(response.body.index(track_a.title)).to be < response.body.index(track_z.title)
    end

    it "falls back to disc_number for invalid sort column" do
      album = create(:album)
      track_b = create(:track, album: album, disc_number: 1, track_number: 2, title: "Beta")
      track_a = create(:track, album: album, disc_number: 1, track_number: 1, title: "Alpha")

      get album_path(album, sort: "nonexistent_column")

      expect(response).to have_http_status(:ok)
      expect(response.body.index(track_a.title)).to be < response.body.index(track_b.title)
    end

    it "paginates when more than 20 tracks" do
      album = create(:album)
      21.times { |i| create(:track, album: album, track_number: i + 1, title: "Track #{(i + 1).to_s.rjust(3, '0')}") }

      get album_path(album)

      expect(response.body).to include("page=2")
    end

    it "does not show pagination nav with 20 or fewer tracks" do
      album = create(:album)
      5.times { |i| create(:track, album: album, track_number: i + 1) }

      get album_path(album)

      expect(response.body).not_to include("page=2")
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
