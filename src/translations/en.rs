use makepad_widgets::*;

live_design! {
    link translator_en;
    
    pub SIDEBAR_TAB_DISCOVER = "Discover"
    pub SIDEBAR_TAB_CHAT = "Chat"
    pub SIDEBAR_TAB_MY_MODELS = "My Models"
    pub SIDEBAR_TAB_SETTINGS = "Settings"

    pub MY_MODELS_TITLE = "My Models"
    pub MY_MODELS_NO_MODELS = "You haven't downloaded any models yet."
    pub MY_MODELS_BUTTON_CHANGE_FOLDER = "Change Folder"
    pub MY_MODELS_BUTTON_SHOW_FILES = "Show in Files"

    pub CHAT_AVATAR_ASSISTANT = "P"
    pub CHAT_AVATAR_USER = "U"
    pub CHAT_BUTTON_SAVE = "Save"
    pub CHAT_BUTTON_CANCEL = "Cancel"
    pub CHAT_BUTTON_SEND = "Send"

    pub MODEL_LIST_QUANTIZATION = "Quantization"
    pub MODEL_INFO_SIZE = "Model Size"
    pub MODEL_INFO_ARCHITECTURE = "Architecture"
    pub MODEL_INFO_REQUIRES = "Requires"

    pub LANDING_SEARCH_TITLE = "Discover, download, and run local LLMs"
    pub LANDING_SEARCH_PLACEHOLDER = "Search models..."

    pub SETTINGS_DOWNLOADS_TITLE = "Download Location"
    pub SETTINGS_DOWNLOADS_BUTTON = "Change Folder"
    pub SETTINGS_DOWNLOADS_SHOW = "Show in Files"

    // Settings Screen
    pub SETTINGS_TITLE = "Settings"
    pub SETTINGS_SERVER_INFO = "Local inference server information"
    pub SETTINGS_NO_MODEL_INFO = "Local inference options will appear once you have a model loaded."
    pub SETTINGS_PORT_NUMBER = "Port number:"
    pub SETTINGS_PORT_ERROR = "Something went wrong while loading the model using this port number. Please try another one."
    pub SETTINGS_CODE_EXAMPLE = "Client code example"
}
