class FavoriteResource < Madmin::Resource
  attribute :id, form: false
  attribute :user
  attribute :favorable
  attribute :created_at, form: false
  attribute :updated_at, form: false, index: false

  def self.default_sort_column = "created_at"
  def self.default_sort_direction = "desc"
end
