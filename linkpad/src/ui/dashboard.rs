use makepad_components::makepad_widgets::*;

live_design! {
    use link::widgets::*;
    use crate::ui::style::*;
    use makepad_components::button::*;
    use makepad_components::card::*;
    use makepad_components::dropdown::*;
    use makepad_components::input::*;
    use makepad_components::layout::*;
    use makepad_components::switch::*;

    ProxyOptionCard = <View> {
        width: Fill,
        height: 88,
        flow: Right,
        align: {y: 0.5},
        spacing: (SPACE_2),
        padding: {left: (SPACE_2), right: (SPACE_2), top: (SPACE_2), bottom: (SPACE_2)},
        show_bg: true,
        draw_bg: {color: (PANEL_BG)},

        <View> {
            width: Fill,
            height: Fit,
            flow: Down,
            spacing: (SPACE_1),

            proxy_name = <Label> {
                text: "Proxy Name"
                draw_text: {text_style: <APP_FONT_BODY>{}}
            }
            proxy_meta = <Label> {
                text: "Protocol | UDP"
                draw_text: {text_style: <APP_FONT_CAPTION>{}}
            }
        }

        proxy_speed = <Label> {
            text: "0 ms"
            draw_text: {text_style: <APP_FONT_CAPTION>{}}
        }
        proxy_select_btn = <MpButtonSmall> { text: "Use" }
    }

    ProxyGroupAccordion = <View> {
        width: Fill,
        height: Fit,
        flow: Down,
        spacing: (SPACE_2),
        padding: {left: (SPACE_2), right: (SPACE_2), top: (SPACE_2), bottom: (SPACE_2)},
        show_bg: true,
        draw_bg: {color: (PANEL_ACCENT_BG)},

        header = <View> {
            width: Fill,
            height: Fit,
            flow: Right,
            align: {y: 0.5},
            spacing: (SPACE_2),

            <View> {
                width: Fill,
                height: Fit,
                flow: Down,
                spacing: (SPACE_1),

                group_name = <Label> {
                    text: "Group Name"
                    draw_text: {text_style: <APP_FONT_BODY>{}}
                }
                group_meta = <Label> {
                    text: "type | size"
                    draw_text: {text_style: <APP_FONT_CAPTION>{}}
                }
                group_status = <Label> {
                    text: "Selected: -"
                    draw_text: {text_style: <APP_FONT_CAPTION>{}}
                }
            }

            group_test_btn = <MpButtonSmall> { text: "Test" }
            group_locate_btn = <MpButtonSmall> { text: "Locate" }
            group_open_btn = <MpButtonSmall> { text: "Open" }
        }

        details = <View> {
            width: Fill,
            height: Fit,
            flow: Down,
            spacing: (SPACE_2),

            detail_empty = <Label> {
                text: ""
                draw_text: {text_style: <APP_FONT_CAPTION>{}}
            }

            options_grid = <View> {
                width: Fill,
                height: Fit,
                flow: Right,
                spacing: (SPACE_2),

                left_col = <View> {
                    width: Fill,
                    height: Fit,
                    flow: Down,
                    spacing: (SPACE_2),
                    option_1 = <ProxyOptionCard> {}
                    option_2 = <ProxyOptionCard> {}
                    option_3 = <ProxyOptionCard> {}
                    option_4 = <ProxyOptionCard> {}
                    option_5 = <ProxyOptionCard> {}
                    option_6 = <ProxyOptionCard> {}
                    option_7 = <ProxyOptionCard> {}
                    option_8 = <ProxyOptionCard> {}
                    option_9 = <ProxyOptionCard> {}
                    option_10 = <ProxyOptionCard> {}
                    option_11 = <ProxyOptionCard> {}
                    option_12 = <ProxyOptionCard> {}
                    option_13 = <ProxyOptionCard> {}
                    option_14 = <ProxyOptionCard> {}
                    option_15 = <ProxyOptionCard> {}
                    option_16 = <ProxyOptionCard> {}
                    option_17 = <ProxyOptionCard> {}
                    option_18 = <ProxyOptionCard> {}
                    option_19 = <ProxyOptionCard> {}
                    option_20 = <ProxyOptionCard> {}
                    option_21 = <ProxyOptionCard> {}
                    option_22 = <ProxyOptionCard> {}
                    option_23 = <ProxyOptionCard> {}
                    option_24 = <ProxyOptionCard> {}
                    option_25 = <ProxyOptionCard> {}
                    option_26 = <ProxyOptionCard> {}
                    option_27 = <ProxyOptionCard> {}
                    option_28 = <ProxyOptionCard> {}
                    option_29 = <ProxyOptionCard> {}
                    option_30 = <ProxyOptionCard> {}
                    option_31 = <ProxyOptionCard> {}
                    option_32 = <ProxyOptionCard> {}
                    option_33 = <ProxyOptionCard> {}
                    option_34 = <ProxyOptionCard> {}
                    option_35 = <ProxyOptionCard> {}
                    option_36 = <ProxyOptionCard> {}
                    option_37 = <ProxyOptionCard> {}
                    option_38 = <ProxyOptionCard> {}
                    option_39 = <ProxyOptionCard> {}
                    option_40 = <ProxyOptionCard> {}
                    option_41 = <ProxyOptionCard> {}
                    option_42 = <ProxyOptionCard> {}
                    option_43 = <ProxyOptionCard> {}
                    option_44 = <ProxyOptionCard> {}
                    option_45 = <ProxyOptionCard> {}
                    option_46 = <ProxyOptionCard> {}
                    option_47 = <ProxyOptionCard> {}
                    option_48 = <ProxyOptionCard> {}
                    option_49 = <ProxyOptionCard> {}
                    option_50 = <ProxyOptionCard> {}
                    option_51 = <ProxyOptionCard> {}
                    option_52 = <ProxyOptionCard> {}
                    option_53 = <ProxyOptionCard> {}
                    option_54 = <ProxyOptionCard> {}
                    option_55 = <ProxyOptionCard> {}
                    option_56 = <ProxyOptionCard> {}
                    option_57 = <ProxyOptionCard> {}
                    option_58 = <ProxyOptionCard> {}
                    option_59 = <ProxyOptionCard> {}
                    option_60 = <ProxyOptionCard> {}
                    option_61 = <ProxyOptionCard> {}
                    option_62 = <ProxyOptionCard> {}
                    option_63 = <ProxyOptionCard> {}
                    option_64 = <ProxyOptionCard> {}
                }

                right_col = <View> {
                    width: Fill,
                    height: Fit,
                    flow: Down,
                    spacing: (SPACE_2),
                    option_1 = <ProxyOptionCard> {}
                    option_2 = <ProxyOptionCard> {}
                    option_3 = <ProxyOptionCard> {}
                    option_4 = <ProxyOptionCard> {}
                    option_5 = <ProxyOptionCard> {}
                    option_6 = <ProxyOptionCard> {}
                    option_7 = <ProxyOptionCard> {}
                    option_8 = <ProxyOptionCard> {}
                    option_9 = <ProxyOptionCard> {}
                    option_10 = <ProxyOptionCard> {}
                    option_11 = <ProxyOptionCard> {}
                    option_12 = <ProxyOptionCard> {}
                    option_13 = <ProxyOptionCard> {}
                    option_14 = <ProxyOptionCard> {}
                    option_15 = <ProxyOptionCard> {}
                    option_16 = <ProxyOptionCard> {}
                    option_17 = <ProxyOptionCard> {}
                    option_18 = <ProxyOptionCard> {}
                    option_19 = <ProxyOptionCard> {}
                    option_20 = <ProxyOptionCard> {}
                    option_21 = <ProxyOptionCard> {}
                    option_22 = <ProxyOptionCard> {}
                    option_23 = <ProxyOptionCard> {}
                    option_24 = <ProxyOptionCard> {}
                    option_25 = <ProxyOptionCard> {}
                    option_26 = <ProxyOptionCard> {}
                    option_27 = <ProxyOptionCard> {}
                    option_28 = <ProxyOptionCard> {}
                    option_29 = <ProxyOptionCard> {}
                    option_30 = <ProxyOptionCard> {}
                    option_31 = <ProxyOptionCard> {}
                    option_32 = <ProxyOptionCard> {}
                    option_33 = <ProxyOptionCard> {}
                    option_34 = <ProxyOptionCard> {}
                    option_35 = <ProxyOptionCard> {}
                    option_36 = <ProxyOptionCard> {}
                    option_37 = <ProxyOptionCard> {}
                    option_38 = <ProxyOptionCard> {}
                    option_39 = <ProxyOptionCard> {}
                    option_40 = <ProxyOptionCard> {}
                    option_41 = <ProxyOptionCard> {}
                    option_42 = <ProxyOptionCard> {}
                    option_43 = <ProxyOptionCard> {}
                    option_44 = <ProxyOptionCard> {}
                    option_45 = <ProxyOptionCard> {}
                    option_46 = <ProxyOptionCard> {}
                    option_47 = <ProxyOptionCard> {}
                    option_48 = <ProxyOptionCard> {}
                    option_49 = <ProxyOptionCard> {}
                    option_50 = <ProxyOptionCard> {}
                    option_51 = <ProxyOptionCard> {}
                    option_52 = <ProxyOptionCard> {}
                    option_53 = <ProxyOptionCard> {}
                    option_54 = <ProxyOptionCard> {}
                    option_55 = <ProxyOptionCard> {}
                    option_56 = <ProxyOptionCard> {}
                    option_57 = <ProxyOptionCard> {}
                    option_58 = <ProxyOptionCard> {}
                    option_59 = <ProxyOptionCard> {}
                    option_60 = <ProxyOptionCard> {}
                    option_61 = <ProxyOptionCard> {}
                    option_62 = <ProxyOptionCard> {}
                    option_63 = <ProxyOptionCard> {}
                    option_64 = <ProxyOptionCard> {}
                }
            }

            detail_overflow = <Label> {
                text: ""
                draw_text: {text_style: <APP_FONT_CAPTION>{}}
            }
        }
    }
    pub Dashboard = <View> {
        width: Fill,
        height: Fill,
        flow: Down,
        spacing: (SPACE_4),
        padding: (SPACE_4),

        content_panel = <MpLayoutContent> {
            width: Fill,
            height: Fill,
            padding: (SPACE_4),
            flow: Down,
            spacing: (SPACE_3),
            draw_bg: {color: (PANEL_BG)},

            content_body = <ScrollYView> {
                width: Fill,
                height: Fill,
                flow: Down,
                spacing: (SPACE_3),
                scroll_bars: {
                    show_scroll_x: false,
                    show_scroll_y: true
                }

                profiles_section = <View> {
                    width: Fill,
                    height: Fit,
                    flow: Down,
                    spacing: (SPACE_3),

                    profiles_import_card = <MpCard> {
                        width: Fill,
                        <MpCardContent> {
                            width: Fill,
                            flow: Down,
                            spacing: (SPACE_1),

                            profile_url_label = <Label> {
                                text: "Profile URL"
                                draw_text: {text_style: <APP_FONT_BODY>{}}
                            }

                            profile_url_input = <MpInput> {
                                width: Fill
                                empty_text: "https://example.com/profile.yaml"
                            }

                            profile_import_status = <Label> {
                                text: "Ready to import profile URL."
                                draw_text: {text_style: <APP_FONT_CAPTION>{}}
                            }

                            <View> {
                                width: Fill,
                                height: Fit,
                                flow: Right,
                                align: {x: 1.0, y: 0.5},

                                profile_import_btn = <MpButtonPrimary> {
                                    text: "Validate & Import"
                                }
                            }
                        }
                    }

                    current_profile_card = <MpCard> {
                        visible: false,
                        width: Fill,
                        <MpCardHeader> {
                            current_profile_title = <MpCardTitle> { text: "Current Profile" }
                        }
                        <MpCardContent> {
                            width: Fill,
                            flow: Down,
                            spacing: (SPACE_1),

                            current_profile_name = <Label> { text: "Name: -" draw_text: {text_style: <APP_FONT_BODY>{}} }
                            current_profile_source = <Label> { text: "Source: -" draw_text: {text_style: <APP_FONT_BODY>{}} }
                            current_profile_updated = <Label> { text: "Updated: -" draw_text: {text_style: <APP_FONT_BODY>{}} }
                            current_profile_stats = <Label> { text: "Stats: nodes 0 | groups 0 | rules 0" draw_text: {text_style: <APP_FONT_BODY>{}} }
                            current_profile_empty = <Label> { text: "No active profile yet." draw_text: {text_style: <APP_FONT_CAPTION>{}} }
                        }
                    }

                    profiles_list_card = <MpCard> {
                        width: Fill,
                        <MpCardHeader> {
                            profiles_list_title = <MpCardTitle> { text: "Profiles" }
                        }
                        <MpCardContent> {
                            width: Fill,
                            flow: Down,
                            spacing: (SPACE_2),

                            profiles_empty_label = <Label> { text: "No profiles imported." draw_text: {text_style: <APP_FONT_CAPTION>{}} }

                            profile_row_1 = <View> {
                                width: Fill,
                                height: Fit,
                                flow: Right,
                                align: {y: 1.0},
                                spacing: (SPACE_2),
                                padding: {left: (SPACE_2), right: (SPACE_2), top: (SPACE_2), bottom: (SPACE_2)},
                                show_bg: true,
                                draw_bg: {color: (PANEL_ACCENT_BG)},

                                <View> {
                                    width: Fill,
                                    height: Fit,
                                    flow: Down,
                                    spacing: (SPACE_1),
                                    profile_row_1_name = <Label> { text: "Profile Name" draw_text: {text_style: <APP_FONT_BODY>{}} }
                                    profile_row_1_meta = <Label> {
                                        width: Fill
                                        text: "source / updated"
                                        draw_text: {text_style: <APP_FONT_CAPTION>{}, wrap: Word}
                                    }
                                    profile_row_1_status = <Label> { text: "Active" draw_text: {text_style: <APP_FONT_CAPTION>{}} }
                                }
                                <View> {
                                    width: Fit,
                                    height: Fit,
                                    flow: Right,
                                    spacing: (SPACE_1),
                                    profile_row_1_activate_btn = <MpButtonSmall> { text: "Activate" }
                                    profile_row_1_refresh_btn = <MpButtonSmall> { text: "Refresh" }
                                    profile_row_1_delete_btn = <MpButtonSmall> { text: "Delete" }
                                }
                            }

                            profile_row_2 = <View> {
                                width: Fill,
                                height: Fit,
                                flow: Right,
                                align: {y: 1.0},
                                spacing: (SPACE_2),
                                padding: {left: (SPACE_2), right: (SPACE_2), top: (SPACE_2), bottom: (SPACE_2)},
                                show_bg: true,
                                draw_bg: {color: (PANEL_ACCENT_BG)},

                                <View> {
                                    width: Fill,
                                    height: Fit,
                                    flow: Down,
                                    spacing: (SPACE_1),
                                    profile_row_2_name = <Label> { text: "Profile Name" draw_text: {text_style: <APP_FONT_BODY>{}} }
                                    profile_row_2_meta = <Label> {
                                        width: Fill
                                        text: "source / updated"
                                        draw_text: {text_style: <APP_FONT_CAPTION>{}, wrap: Word}
                                    }
                                    profile_row_2_status = <Label> { text: "Inactive" draw_text: {text_style: <APP_FONT_CAPTION>{}} }
                                }
                                <View> {
                                    width: Fit,
                                    height: Fit,
                                    flow: Right,
                                    spacing: (SPACE_1),
                                    profile_row_2_activate_btn = <MpButtonSmall> { text: "Activate" }
                                    profile_row_2_refresh_btn = <MpButtonSmall> { text: "Refresh" }
                                    profile_row_2_delete_btn = <MpButtonSmall> { text: "Delete" }
                                }
                            }

                            profile_row_3 = <View> {
                                width: Fill,
                                height: Fit,
                                flow: Right,
                                align: {y: 1.0},
                                spacing: (SPACE_2),
                                padding: {left: (SPACE_2), right: (SPACE_2), top: (SPACE_2), bottom: (SPACE_2)},
                                show_bg: true,
                                draw_bg: {color: (PANEL_ACCENT_BG)},

                                <View> {
                                    width: Fill,
                                    height: Fit,
                                    flow: Down,
                                    spacing: (SPACE_1),
                                    profile_row_3_name = <Label> { text: "Profile Name" draw_text: {text_style: <APP_FONT_BODY>{}} }
                                    profile_row_3_meta = <Label> {
                                        width: Fill
                                        text: "source / updated"
                                        draw_text: {text_style: <APP_FONT_CAPTION>{}, wrap: Word}
                                    }
                                    profile_row_3_status = <Label> { text: "Inactive" draw_text: {text_style: <APP_FONT_CAPTION>{}} }
                                }
                                <View> {
                                    width: Fit,
                                    height: Fit,
                                    flow: Right,
                                    spacing: (SPACE_1),
                                    profile_row_3_activate_btn = <MpButtonSmall> { text: "Activate" }
                                    profile_row_3_refresh_btn = <MpButtonSmall> { text: "Refresh" }
                                    profile_row_3_delete_btn = <MpButtonSmall> { text: "Delete" }
                                }
                            }
                        }
                    }
                }

                proxy_groups_section = <View> {
                    width: Fill,
                    height: Fit,
                    flow: Down,
                    spacing: (SPACE_3),

                    proxy_groups_card = <MpCard> {
                        width: Fill,
                        <MpCardHeader> {
                            <View> {
                                width: Fill,
                                height: Fit,
                                flow: Right,
                                align: {x: 1.0, y: 0.5},
                                spacing: (SPACE_2),

                                <View> {width: Fill, height: Fit}
                                proxy_mode_rule_btn = <MpButtonSmall> { text: "Rule" }
                                proxy_mode_global_btn = <MpButtonSmall> { text: "Global" }
                                proxy_mode_direct_btn = <MpButtonSmall> { text: "Direct" }
                            }
                        }
                        <MpCardContent> {
                            width: Fill,
                            flow: Down,
                            spacing: (SPACE_1),

                            proxy_groups_empty = <Label> { text: "No proxy groups in active profile." draw_text: {text_style: <APP_FONT_CAPTION>{}} }

                            proxy_group_row_1 = <ProxyGroupAccordion> {}
                            proxy_group_row_2 = <ProxyGroupAccordion> {}
                            proxy_group_row_3 = <ProxyGroupAccordion> {}
                            proxy_group_row_4 = <ProxyGroupAccordion> {}
                            proxy_group_row_5 = <ProxyGroupAccordion> {}
                            proxy_group_row_6 = <ProxyGroupAccordion> {}
                            proxy_group_row_7 = <ProxyGroupAccordion> {}
                            proxy_group_row_8 = <ProxyGroupAccordion> {}
                        }
                    }
                }

                rules_section = <View> {
                    width: Fill,
                    height: Fit,
                    flow: Down,
                    spacing: (SPACE_3),

                    rules_card = <MpCard> {
                        width: Fill,
                        <MpCardHeader> {
                            rules_title = <MpCardTitle> { text: "Rules" }
                            rules_desc = <MpCardDescription> { text: "Rules from the active profile." }
                        }
                        <MpCardContent> {
                            width: Fill,
                            flow: Down,
                            spacing: (SPACE_2),

                            rules_search_input = <MpInput> {
                                width: Fill
                                empty_text: "Search rules"
                            }

                            <View> {
                                width: Fill,
                                height: Fit,
                                flow: Right,
                                spacing: (SPACE_2),

                                rules_filter_all_btn = <MpButtonSmall> { text: "All" }
                                rules_filter_domain_btn = <MpButtonSmall> { text: "Domain" }
                                rules_filter_ip_cidr_btn = <MpButtonSmall> { text: "IP-CIDR" }
                                rules_filter_process_btn = <MpButtonSmall> { text: "Process" }
                            }

                            rules_count = <Label> {
                                text: "Total rules: 0"
                                draw_text: {text_style: <APP_FONT_BODY>{}}
                            }
                            rules_empty = <Label> {
                                text: "No rules in active profile."
                                draw_text: {text_style: <APP_FONT_CAPTION>{}}
                            }
                            rules_list = <Label> {
                                text: ""
                                draw_text: {text_style: <APP_FONT_CAPTION>{}}
                            }
                        }
                    }
                }

                settings_section = <View> {
                    width: Fill,
                    height: Fit,
                    flow: Down,
                    spacing: (SPACE_3),

                    basic_settings_card = <MpCard> {
                        width: Fill,
                        <MpCardHeader> {
                            basic_setting_title = <MpCardTitle> { text: "Linkpad Basic Setting" }
                        }

                        <MpCardContent> {
                            width: Fill,
                            flow: Down,
                            spacing: (SPACE_3),

                            <View> {
                                width: Fill,
                                height: Fit,
                                flow: Right,
                                align: {y: 0.5},
                                spacing: (SPACE_3),

                                language_label = <Label> {text: "Language", draw_text: {text_style: <APP_FONT_BODY>{}, color: (TEXT_PRIMARY)}}
                                <View> {width: Fill, height: Fit}
                                language_dropdown = <MpDropdown> {
                                    width: 200,
                                    labels: ["English", "简体中文"],
                                    selected_item: 0
                                }
                            }

                            <View> {
                                width: Fill,
                                height: Fit,
                                flow: Right,
                                align: {y: 0.5},
                                spacing: (SPACE_3),

                                theme_label = <Label> {text: "Theme", draw_text: {text_style: <APP_FONT_BODY>{}, color: (TEXT_PRIMARY)}}
                                <View> {width: Fill, height: Fit}
                                theme_dropdown = <MpDropdown> {
                                    width: 200,
                                    labels: ["Light", "Dark", "System"],
                                    selected_item: 2
                                }
                            }
                        }
                    }

                    system_settings_card = <MpCard> {
                        width: Fill,
                        <MpCardHeader> {
                            system_setting_title = <MpCardTitle> { text: "System Setting" }
                        }
                        <MpCardContent> {
                            width: Fill,
                            flow: Down,
                            spacing: (SPACE_3),

                            <View> {
                                width: Fill,
                                height: Fit,
                                flow: Right,
                                align: {y: 0.5},
                                spacing: (SPACE_3),

                                system_proxy_label = <Label> {text: "System Proxy", draw_text: {text_style: <APP_FONT_BODY>{}, color: (TEXT_PRIMARY)}}
                                <View> {width: Fill, height: Fit}
                                system_proxy_switch = <MpSwitch> {}
                            }

                            <View> {
                                width: Fill,
                                height: Fit,
                                flow: Right,
                                align: {y: 0.5},
                                spacing: (SPACE_3),

                                auto_launch_label = <Label> {text: "Auto Launch", draw_text: {text_style: <APP_FONT_BODY>{}, color: (TEXT_PRIMARY)}}
                                <View> {width: Fill, height: Fit}
                                auto_launch_switch = <MpSwitch> {}
                            }

                            <View> {
                                width: Fill,
                                height: Fit,
                                flow: Right,
                                align: {y: 0.5},
                                spacing: (SPACE_3),

                                silent_start_label = <Label> {text: "Silent Start", draw_text: {text_style: <APP_FONT_BODY>{}, color: (TEXT_PRIMARY)}}
                                <View> {width: Fill, height: Fit}
                                silent_start_switch = <MpSwitch> {}
                            }
                        }
                    }

                    clash_settings_card = <MpCard> {
                        width: Fill,
                        <MpCardHeader> {
                            <View> {
                                width: Fill,
                                height: Fit,
                                flow: Right,
                                align: {y: 0.5},
                                spacing: (SPACE_2),

                                clash_setting_title = <MpCardTitle> { text: "Clash Setting" }
                                <View> {width: Fill, height: Fit}
                                clash_core_upgrade_btn = <MpButtonPrimary> { text: "UPGRADE" }
                                clash_core_restart_btn = <MpButtonPrimary> { text: "RESTART" }
                            }
                        }
                        <MpCardContent> {
                            width: Fill,
                            flow: Down,
                            spacing: (SPACE_3),

                            <View> {
                                width: Fill,
                                height: Fit,
                                flow: Right,
                                align: {y: 0.5},
                                spacing: (SPACE_3),

                                clash_port_label = <Label> {text: "Port Config", draw_text: {text_style: <APP_FONT_BODY>{}, color: (TEXT_PRIMARY)}}
                                <View> {width: Fill, height: Fit}
                                clash_port_input = <MpInput> {
                                    width: 120
                                    empty_text: "7890"
                                }
                                clash_port_save_btn = <MpButtonPrimary> {
                                    text: "Save"
                                }
                            }

                            <View> {
                                width: Fill,
                                height: Fit,
                                flow: Right,
                                align: {y: 0.5},
                                spacing: (SPACE_3),

                                clash_core_version_label = <Label> {text: "Clash Core Version", draw_text: {text_style: <APP_FONT_BODY>{}, color: (TEXT_PRIMARY)}}
                                <View> {width: Fill, height: Fit}
                                clash_core_version_value = <Label> {
                                    text: "Unknown"
                                    draw_text: {text_style: <APP_FONT_BODY>{}, color: (TEXT_MUTED)}
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
