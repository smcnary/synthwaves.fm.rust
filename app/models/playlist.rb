class Playlist < ApplicationRecord
  belongs_to :user
  has_many :playlist_tracks, -> { order(:position) }, dependent: :destroy
  has_many :tracks, through: :playlist_tracks

  validates :name, presence: true

  def random_cover_track
    tracks.joins(album: :cover_image_attachment).order("RANDOM()").first
  end

  SORT_OPTIONS = {
    "name" => "Name",
    "playlist_tracks_count" => "Track Count",
    "updated_at" => "Recently Updated",
    "created_at" => "Recently Created"
  }.freeze

  scope :search, ->(query) {
    where("playlists.name LIKE :q", q: "%#{query}%") if query.present?
  }
end
