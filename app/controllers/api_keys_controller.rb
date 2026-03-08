class APIKeysController < ApplicationController
  before_action :set_api_key, only: [:destroy]

  def index
    @api_keys = Current.user.api_keys.order(created_at: :desc)
  end

  def new
    @api_key = Current.user.api_keys.build
  end

  def create
    @api_key = Current.user.api_keys.build(api_key_params)
    secret_key = SecureRandom.hex(32)
    @api_key.secret_key = secret_key

    if @api_key.save
      flash[:api_secret] = secret_key
      redirect_to api_keys_path, notice: "API key created. Copy your secret key now - it won't be shown again."
    else
      render :new, status: :unprocessable_content
    end
  end

  def destroy
    @api_key.destroy
    redirect_to api_keys_path, notice: "API key revoked."
  end

  private

  def set_api_key
    @api_key = Current.user.api_keys.find(params[:id])
  end

  def api_key_params
    params.require(:api_key).permit(:name, :expires_at)
  end
end
