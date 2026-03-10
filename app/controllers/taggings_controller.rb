class TaggingsController < ApplicationController
  def create
    tag = Tag.find_or_create_by!(name: tag_params[:name].strip.downcase, tag_type: tag_params[:tag_type])
    @tagging = Current.user.taggings.create!(
      tag: tag,
      taggable_type: tag_params[:taggable_type],
      taggable_id: tag_params[:taggable_id]
    )

    respond_to do |format|
      format.turbo_stream { render turbo_stream: turbo_stream.replace("tags-#{tag_params[:taggable_type].downcase}-#{tag_params[:taggable_id]}", partial: "taggings/tags", locals: {taggable: @tagging.taggable}) }
      format.html { redirect_back fallback_location: root_path }
    end
  end

  def destroy
    @tagging = Current.user.taggings.find(params[:id])
    taggable = @tagging.taggable
    @tagging.destroy!

    respond_to do |format|
      format.turbo_stream { render turbo_stream: turbo_stream.replace("tags-#{taggable.class.name.downcase}-#{taggable.id}", partial: "taggings/tags", locals: {taggable: taggable}) }
      format.html { redirect_back fallback_location: root_path }
    end
  end

  private

  def tag_params
    params.require(:tagging).permit(:name, :tag_type, :taggable_type, :taggable_id)
  end
end
