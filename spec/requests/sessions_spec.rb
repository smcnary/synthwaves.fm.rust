require "rails_helper"

RSpec.describe "Sessions", type: :request do
  describe "DELETE /session" do
    it "resets theme to default after logout" do
      user = create(:user, theme: "punk")
      login_user(user)

      delete "/session"

      expect(response).to redirect_to(new_session_path)
      follow_redirect!

      expect(response.body).to include('data-theme="synthwave"')
    end
  end
end
