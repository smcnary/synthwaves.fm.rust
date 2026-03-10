require "rails_helper"

RSpec.describe Tag, type: :model do
  describe "validations" do
    subject { build(:tag) }

    it { is_expected.to validate_presence_of(:name) }
    it { is_expected.to validate_presence_of(:tag_type) }
    it { is_expected.to validate_uniqueness_of(:name).scoped_to(:tag_type) }
    it { is_expected.to validate_inclusion_of(:tag_type).in_array(%w[genre mood]) }
  end

  describe "associations" do
    it { is_expected.to have_many(:taggings).dependent(:destroy) }
  end

  describe "scopes" do
    it "filters by genre" do
      genre_tag = create(:tag, tag_type: "genre")
      create(:tag, tag_type: "mood")

      expect(Tag.genres).to eq([genre_tag])
    end

    it "filters by mood" do
      create(:tag, tag_type: "genre")
      mood_tag = create(:tag, tag_type: "mood")

      expect(Tag.moods).to eq([mood_tag])
    end
  end
end
