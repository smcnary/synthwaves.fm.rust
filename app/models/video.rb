class Video < ApplicationRecord
  belongs_to :user
  has_one_attached :file
  has_one_attached :thumbnail
  has_many :favorites, as: :favorable, dependent: :destroy

  validates :title, presence: true

  scope :ready, -> { where(status: "ready") }
  scope :search, ->(q) { q.present? ? where("title LIKE ?", "%#{q}%") : all }

  after_create_commit :convert_video

  private

  def convert_video
    VideoConversionJob.perform_later(id)
  end
end
