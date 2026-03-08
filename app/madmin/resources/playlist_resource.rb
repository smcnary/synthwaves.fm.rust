class PlaylistResource < Madmin::Resource
  attribute :id, form: false
  attribute :name
  attribute :user
  attribute :playlist_tracks, index: false
  attribute :tracks, index: false
  attribute :created_at, form: false, index: false
  attribute :updated_at, form: false, index: false

  def self.display_name(record) = record.name
  def self.default_sort_column = "name"
end
