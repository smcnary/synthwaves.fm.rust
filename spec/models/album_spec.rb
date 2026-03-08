require "rails_helper"

RSpec.describe Album, type: :model do
  describe "associations" do
    it { should belong_to(:artist) }
    it { should have_many(:tracks).dependent(:destroy) }
    it { should have_many(:favorites).dependent(:destroy) }
    it { should have_one_attached(:cover_image) }
  end

  describe "validations" do
    subject { build(:album) }

    it { should validate_presence_of(:title) }
    it { should validate_uniqueness_of(:title).scoped_to(:artist_id) }
  end

  describe "category scopes" do
    let!(:music_album) { create(:album, artist: create(:artist, category: "music")) }
    let!(:podcast_album) { create(:album, artist: create(:artist, :podcast)) }

    it ".music returns only albums belonging to music artists" do
      expect(Album.music).to include(music_album)
      expect(Album.music).not_to include(podcast_album)
    end

    it ".podcast returns only albums belonging to podcast artists" do
      expect(Album.podcast).to include(podcast_album)
      expect(Album.podcast).not_to include(music_album)
    end
  end
end
