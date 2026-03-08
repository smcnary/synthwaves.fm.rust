class Artist < ApplicationRecord
  has_many :albums, dependent: :destroy
  has_many :tracks, dependent: :destroy
  has_many :favorites, as: :favorable, dependent: :destroy

  enum :category, { music: "music", podcast: "podcast" }, default: "music"

  CATEGORIES = categories.keys

  validates :name, presence: true, uniqueness: true
end
