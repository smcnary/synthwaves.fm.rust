FactoryBot.define do
  factory :radio_station do
    user
    sequence(:name) { |n| "Radio Station #{n}" }
    youtube_url { "https://www.youtube.com/watch?v=jfKfPfyJRdk" }
    youtube_video_id { "jfKfPfyJRdk" }
  end
end
