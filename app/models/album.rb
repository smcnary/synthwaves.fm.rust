class Album < ApplicationRecord
  belongs_to :artist
  has_many :tracks, dependent: :destroy
  has_one_attached :cover_image
  has_many :favorites, as: :favorable, dependent: :destroy

  validates :title, presence: true, uniqueness: {scope: :artist_id}

  scope :music, -> { joins(:artist).merge(Artist.music) }
  scope :podcast, -> { joins(:artist).merge(Artist.podcast) }
end
