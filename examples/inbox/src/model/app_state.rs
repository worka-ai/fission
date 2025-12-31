use serde::{Deserialize, Serialize};
use fission_core::AppState;
use std::collections::HashSet;
use super::email::Folder;
use chrono::NaiveDate;
use fission_i18n::Locale;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InboxState {
    pub locale: Locale,
    // Router
    pub current_path: String,
    
    // ... (existing) ...
    pub selected_folder: Folder,
    pub selected_email_id: Option<usize>,
    pub selected_emails: Vec<usize>,
    
    // List View
    pub page: usize,
    pub total_pages: usize,
    pub filter_mode: usize, // 0=All, 1=Unread, 2=Starred
    
    // Compose State
    pub compose_to: String,
    pub compose_subject: String,
    pub compose_body: String,
    pub compose_attachments: Vec<String>,
    pub schedule_date: Option<NaiveDate>,
    pub schedule_time: Option<(u32, u32)>,
    pub is_date_picker_open: bool,
    pub is_time_picker_open: bool, // Not used by standard TimePicker (inline) but maybe for modal?

    // Filters / Toolbar
    pub sort_option: String,
    pub show_advanced_filters: bool,
    pub zoom_level: f32,

    // Settings / Labs
    pub signature: String,
    pub signature_editing: bool,
    pub smart_compose_enabled: bool,
    pub offline_enabled: bool,
    pub auto_advance_enabled: bool,

    // Right Sidebar / Meet
    pub meet_camera_on: bool,
    pub meet_mic_on: bool,

    // UX
    pub show_help_popover: bool,
    pub last_drag_label: Option<String>,
    pub show_quick_tip: bool,
    
    // UI State
    pub search_query: String,
    pub show_filter_dropdown: bool,
    pub active_tab: usize, 
    pub reply_mode: usize, 
    pub notifications_enabled: bool,
    pub details_expanded: bool,
    pub storage_usage: f32,
    
    // Modals
    pub show_settings: bool,
    pub show_contacts: bool,
    pub show_compose: bool,
    pub show_toast: bool,
    pub show_mobile_menu: bool,
    pub toast_message: Option<String>,
    pub show_browser_demo: bool,
    pub browser_url: String,
    
    // Preferences
    pub theme_mode: String,
    pub density_mode: String,
    
    // Tree View State
    pub expanded_folders: HashSet<String>,
}

impl Default for InboxState {
    fn default() -> Self {
        Self {
            locale: Locale("en-US".into()),
            current_path: "/inbox".into(),
            selected_folder: Folder::Inbox,
            selected_email_id: None,
            selected_emails: vec![],
            
            page: 1,
            total_pages: 5,
            filter_mode: 0,
            
            compose_to: "".into(),
            compose_subject: "".into(),
            compose_body: "".into(),
            compose_attachments: vec![],
            schedule_date: None,
            schedule_time: None,
            is_date_picker_open: false,
            is_time_picker_open: false,
            sort_option: "Newest".into(),
            show_advanced_filters: false,
            zoom_level: 1.0,
            signature: "Best regards,\nFission Team".into(),
            signature_editing: false,
            smart_compose_enabled: true,
            offline_enabled: false,
            auto_advance_enabled: true,
            meet_camera_on: true,
            meet_mic_on: true,
            show_help_popover: false,
            last_drag_label: None,
            show_quick_tip: true,
            
            search_query: "".into(),
            show_filter_dropdown: false,
            active_tab: 0,
            reply_mode: 0,
            notifications_enabled: true,
            details_expanded: true,
            storage_usage: 0.3,
            show_settings: false,
            show_contacts: false,
            show_compose: false,
            show_toast: false,
            show_mobile_menu: false,
            show_browser_demo: false,
            browser_url: "https://example.com".into(),
            toast_message: None,
            theme_mode: "Light".into(),
            density_mode: "Comfortable".into(),
            expanded_folders: HashSet::new(),
        }
    }
}

impl AppState for InboxState {}
