require "rails_helper"

RSpec.describe "API::V1::Native", type: :request do
  describe "GET /api/v1/native/credentials" do
    context "when authenticated" do
      let(:user) { create(:user, subsonic_password: "test_subsonic_pass") }

      before { login_user(user) }

      it "returns the user's credentials" do
        get "/api/v1/native/credentials"

        expect(response).to have_http_status(:ok)
        json = response.parsed_body
        expect(json["email"]).to eq(user.email_address)
        expect(json["subsonic_password"]).to eq("test_subsonic_pass")
        expect(json["theme"]).to eq(user.theme)
      end
    end

    context "when not authenticated" do
      it "redirects to login" do
        get "/api/v1/native/credentials"

        expect(response).to redirect_to(new_session_path)
      end
    end
  end
end
