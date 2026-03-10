require "rails_helper"

RSpec.describe Tagging, type: :model do
  describe "validations" do
    subject { build(:tagging) }

    it { is_expected.to validate_uniqueness_of(:tag_id).scoped_to([:taggable_type, :taggable_id, :user_id]) }
  end

  describe "associations" do
    it { is_expected.to belong_to(:tag) }
    it { is_expected.to belong_to(:taggable) }
    it { is_expected.to belong_to(:user) }
  end
end
