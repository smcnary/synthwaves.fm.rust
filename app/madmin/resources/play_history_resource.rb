class PlayHistoryResource < Madmin::Resource
  attribute :id, form: false
  attribute :user
  attribute :track
  attribute :played_at
  attribute :created_at, form: false, index: false
  attribute :updated_at, form: false, index: false

  def self.default_sort_column = "played_at"
  def self.default_sort_direction = "desc"
end
