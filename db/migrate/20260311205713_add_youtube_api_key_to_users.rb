class AddYoutubeAPIKeyToUsers < ActiveRecord::Migration[8.1]
  def change
    add_column :users, :youtube_api_key, :string
  end
end
