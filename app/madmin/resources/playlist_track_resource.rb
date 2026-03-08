class PlaylistTrackResource < Madmin::Resource
  attribute :id, form: false
  attribute :playlist
  attribute :track
  attribute :position
  attribute :created_at, form: false, index: false
  attribute :updated_at, form: false, index: false

  def self.display_name(record) = "#{record.playlist&.name} ##{record.position}"
  def self.default_sort_column = "position"
end
