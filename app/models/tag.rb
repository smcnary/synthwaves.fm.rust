class Tag < ApplicationRecord
  has_many :taggings, dependent: :destroy

  validates :name, presence: true, uniqueness: {scope: :tag_type}
  validates :tag_type, presence: true, inclusion: {in: %w[genre mood]}

  scope :genres, -> { where(tag_type: "genre") }
  scope :moods, -> { where(tag_type: "mood") }
end
