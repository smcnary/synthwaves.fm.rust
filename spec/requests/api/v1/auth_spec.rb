require "rails_helper"

RSpec.describe "API::V1::Auth", type: :request do
  describe "POST /api/v1/auth/token" do
    let(:user) { create(:user) }
    let(:api_key) { create(:api_key, user: user) }
    let(:secret_key) { "test_secret_key_12345" }

    before do
      api_key.secret_key = secret_key
      api_key.save!
    end

    context "with valid credentials" do
      it "returns a JWT token" do
        post "/api/v1/auth/token", params: {
          client_id: api_key.client_id,
          secret_key: secret_key
        }

        expect(response).to have_http_status(:ok)
        json = JSON.parse(response.body)
        expect(json["token"]).to be_present
        expect(json["expires_in"]).to eq(3600)
      end

      it "updates the API key's last_used_at" do
        freeze_time do
          post "/api/v1/auth/token", params: {
            client_id: api_key.client_id,
            secret_key: secret_key
          }

          api_key.reload
          expect(api_key.last_used_at).to eq(Time.current)
        end
      end

      it "records the client IP address" do
        post "/api/v1/auth/token", params: {
          client_id: api_key.client_id,
          secret_key: secret_key
        }

        api_key.reload
        expect(api_key.last_used_ip).to be_present
      end
    end

    context "with invalid credentials" do
      it "returns unauthorized for wrong secret key" do
        post "/api/v1/auth/token", params: {
          client_id: api_key.client_id,
          secret_key: "wrong_key"
        }

        expect(response).to have_http_status(:unauthorized)
        json = JSON.parse(response.body)
        expect(json["error"]).to eq("Invalid credentials")
      end

      it "returns unauthorized for non-existent client_id" do
        post "/api/v1/auth/token", params: {
          client_id: "non_existent",
          secret_key: secret_key
        }

        expect(response).to have_http_status(:unauthorized)
        json = JSON.parse(response.body)
        expect(json["error"]).to eq("Invalid credentials")
      end
    end

    context "with expired API key" do
      before do
        api_key.update!(expires_at: 1.day.ago)
      end

      it "returns unauthorized" do
        post "/api/v1/auth/token", params: {
          client_id: api_key.client_id,
          secret_key: secret_key
        }

        expect(response).to have_http_status(:unauthorized)
        json = JSON.parse(response.body)
        expect(json["error"]).to eq("API key has expired")
      end
    end
  end
end
