require "rails_helper"

RSpec.describe Playlist, type: :model do
  describe "associations" do
    it { should belong_to(:user) }
    it { should have_many(:playlist_tracks).dependent(:destroy) }
    it { should have_many(:tracks).through(:playlist_tracks) }
  end

  describe "validations" do
    it { should validate_presence_of(:name) }
  end

  describe "#random_cover_track" do
    let(:user) { create(:user) }
    let(:playlist) { create(:playlist, user: user) }

    it "returns nil for an empty playlist" do
      expect(playlist.random_cover_track).to be_nil
    end

    it "returns nil when no albums have cover images" do
      track = create(:track)
      create(:playlist_track, playlist: playlist, track: track)

      expect(playlist.random_cover_track).to be_nil
    end

    it "returns a track whose album has a cover image" do
      album_with_cover = create(:album)
      album_with_cover.cover_image.attach(
        io: StringIO.new("fake image data"),
        filename: "cover.jpg",
        content_type: "image/jpeg"
      )
      track_with_cover = create(:track, album: album_with_cover, artist: album_with_cover.artist)
      create(:playlist_track, playlist: playlist, track: track_with_cover)

      track_without_cover = create(:track)
      create(:playlist_track, playlist: playlist, track: track_without_cover)

      expect(playlist.random_cover_track).to eq(track_with_cover)
    end
  end

  describe ".search" do
    let(:user) { create(:user) }
    let!(:chill) { create(:playlist, user: user, name: "Chill Vibes") }
    let!(:rock) { create(:playlist, user: user, name: "Rock Anthems") }
    let!(:chill_rock) { create(:playlist, user: user, name: "Chill Rock Mix") }

    it "returns all playlists when query is nil" do
      expect(Playlist.search(nil)).to contain_exactly(chill, rock, chill_rock)
    end

    it "returns all playlists when query is blank" do
      expect(Playlist.search("")).to contain_exactly(chill, rock, chill_rock)
    end

    it "filters playlists by name" do
      expect(Playlist.search("Chill")).to contain_exactly(chill, chill_rock)
    end

    it "is case-insensitive" do
      expect(Playlist.search("chill")).to contain_exactly(chill, chill_rock)
    end

    it "returns empty when no match" do
      expect(Playlist.search("Jazz")).to be_empty
    end
  end
end
