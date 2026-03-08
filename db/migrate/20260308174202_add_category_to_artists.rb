class AddCategoryToArtists < ActiveRecord::Migration[8.1]
  def change
    add_column :artists, :category, :string, default: "music", null: false
    add_index :artists, :category
  end
end
