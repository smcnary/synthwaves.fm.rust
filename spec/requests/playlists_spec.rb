require "rails_helper"

RSpec.describe "Playlists", type: :request do
  let(:user) { create(:user) }

  before { login_user(user) }

  describe "GET /playlists" do
    it "returns success" do
      get playlists_path
      expect(response).to have_http_status(:ok)
    end
  end

  describe "POST /playlists" do
    it "creates a playlist" do
      expect {
        post playlists_path, params: {playlist: {name: "My Playlist"}}
      }.to change(Playlist, :count).by(1)
      expect(response).to redirect_to(playlist_path(Playlist.last))
    end

    it "rejects blank name" do
      post playlists_path, params: {playlist: {name: ""}}
      expect(response).to have_http_status(:unprocessable_content)
    end
  end

  describe "PATCH /playlists/:id" do
    let(:playlist) { create(:playlist, user: user) }

    it "updates the playlist" do
      patch playlist_path(playlist), params: {playlist: {name: "Updated"}}
      expect(playlist.reload.name).to eq("Updated")
    end
  end

  describe "GET /playlists/:id" do
    let(:playlist) { create(:playlist, user: user) }
    let(:track) { create(:track) }

    before do
      create(:playlist_track, playlist: playlist, track: track, position: 1)
      get playlist_path(playlist)
    end

    it "renders song-row controller on each track" do
      expect(response.body).to include('data-controller="song-row"')
    end

    it "renders play button with correct data-action" do
      expect(response.body).to include('data-action="song-row#play"')
    end

    it "renders stream URL value" do
      expect(response.body).to include("data-song-row-stream-url-value")
    end

    it "renders track id value" do
      expect(response.body).to include("data-song-row-track-id-value=\"#{track.id}\"")
    end

    it "does not nest the play button inside a form" do
      doc = Nokogiri::HTML(response.body)
      play_button = doc.at_css('button[data-action="song-row#play"]')
      expect(play_button).to be_present
      expect(play_button.ancestors("form")).to be_empty
    end
  end

  describe "DELETE /playlists/:id" do
    let!(:playlist) { create(:playlist, user: user) }

    it "deletes the playlist" do
      expect {
        delete playlist_path(playlist)
      }.to change(Playlist, :count).by(-1)
    end
  end
end
