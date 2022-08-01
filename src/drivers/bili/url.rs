// region =================== Base ========================

pub(super) const WWW: &str = "https://www.bilibili.com";

/** 主站 */
pub(super) const MAIN: &str = "https://api.bilibili.com";

/** SPACE */
pub(super) const SPACE: &str = "https://space.bilibili.com";

/** 直播中心 */
pub(super) const LIVE: &str = "https://api.live.bilibili.com";

/** 直播 API */
pub(super) const LIVE_API: &str = "https://api.live.bilibili.com";

/** 鉴权 */
pub(super) const PASSPORT: &str = "https://passport.bilibili.com";

pub(super) const ACCOUNT: &str = "https://account.bilibili.com";

pub(super) const VC_API: &str = "https://api.vc.bilibili.com";

pub(super) const FEED_DOMAIN: &str = "https://t.bilibili.com";

// endregion

// region =================== Bangumi ========================

pub(super) const BANGUMI_INFO_GET_URL: &str = "https://api.bilibili.com/pgc/review/user";

pub(super) const BANGUMI_DETAILED_GET_URL: &str = "https://api.bilibili.com/pgc/view/web/season";

pub(super) const BANGUMI_REVIEW_INFO_GET_URL: &str = "https://api.bilibili.com/pgc/review/user";

// endregion

// region =================== Video ========================

pub(super) const VIDEO_INFO_GET_URL: &str = "https://api.bilibili.com/x/web-interface/view";

pub(super) const VIDEO_PLAYER_INFO_GET_URL: &str = "https://api.bilibili.com/x/player/v2";

pub(super) const VIDEO_PARTS_GET_URL: &str = "https://api.bilibili.com/x/player/pagelist";

pub(super) const VIDEO_DESCRIPTION_GET_URL: &str =
  "https://api.bilibili.com/x/web-interface/archive/desc";

pub(super) const VIDEO_LIKE_URL: &str = "https://api.bilibili.com/x/web-interface/archive/like";

pub(super) const VIDEO_HAS_LIKE_URL: &str =
  "https://api.bilibili.com/x/web-interface/archive/has/like";

pub(super) const VIDEO_COIN_URL: &str = "https://api.bilibili.com/x/web-interface/coin/add";

pub(super) const VIDEO_COIN_CHECK_URL: &str =
  "https://api.bilibili.com/x/web-interface/archive/coins";

// pub(super) const VIDEO_COLLECT_ACTION_URL: &str = "https://api.bilibili.com/x/v3/fav/resource/deal";

pub(super) const VIDEO_COLLECT_ACTION_URL: &str =
  "https://api.bilibili.com/medialist/gateway/coll/resource/deal";

pub(super) const VIDEO_COLLECT_CHECK_URL: &str = "https://api.bilibili.com/x/v2/fav/video/favoured";

pub(super) const VIDEO_COMBO_LIKE_URL: &str =
  "https://api.bilibili.com/x/web-interface/archive/like/triple";

pub(super) const VIDEO_SHARE_URL: &str = "https://api.bilibili.com/x/web-interface/share/add";

pub(super) const VIDEO_TIMELINE_HOT_URL: &str = "https://bvc.bilivideo.com/pbp/data";

pub(super) const VIDEO_ONLINE_GET_URL: &str = "https://api.bilibili.com/x/player/online/total";

pub(super) const VIDEO_TAG_GET_URL: &str = "https://api.bilibili.com/x/tag/archive/tags";

pub(super) const VIDEO_RELATED_GET_URL: &str =
  "https://api.bilibili.com/x/web-interface/archive/related";

pub(super) const VIDEO_DANMAKU_WEB_URL: &str = "https://api.bilibili.com/x/v2/dm/web/seg.so";

pub(super) const VIDEO_DANMAKU_CALENDAR_URL: &str =
  "https://api.bilibili.com/x/v2/dm/history/index";

pub(super) const VIDEO_HISTORY_DANMAKU_GET_URL: &str =
  "https://api.bilibili.com/x/v2/dm/web/history/seg.so";

pub(super) const VIDEO_STREAM_FETCH_URL: &str = "https://api.bilibili.com/x/player/playurl";

pub(super) const PGC_STREAM_FETCH_URL: &str = "https://api.bilibili.com/pgc/player/web/playurl";

pub(super) const VIDEO_REPORT_PROGRESS_URL: &str = "https://api.bilibili.com/x/v2/history/report";

pub(super) const VIDEO_INTERACTIVE_INFO_URL: &str = "https://api.bilibili.com/x/stein/edgeinfo_v2";

// endregion

// region =================== Ranking ========================

pub(super) const RANKING_GET_URL: &str = "https://api.bilibili.com/x/web-interface/ranking/region";

pub(super) const LATEST_VIDEO_GET_URL: &str =
  "https://api.bilibili.com/x/web-interface/dynamic/region";

// endregion

// region =================== Time ========================

pub(super) const GET_TIMESTAMP_URL: &str = "https://api.bilibili.com/x/report/click/now";

// endregion

// region =================== Sticker ========================

pub(super) const GET_MY_STICKERS_LIST: &str = "https://api.bilibili.com/x/emote/user/panel/web";

pub(super) const GET_STICKERS_BY_ID_URL: &str = "https://api.bilibili.com/x/emote/package";

pub(super) const GET_ALL_STICKERS_URL: &str = "https://api.bilibili.com/x/emote/setting/panel";

// endregion

// region =================== Info ========================

pub(super) const BASIC_INFO_GET_URL: &str = "https://api.bilibili.com/x/web-interface/nav";

pub(super) const STAT_GET_URL: &str = "https://api.bilibili.com/x/web-interface/nav/stat";

pub(super) const COIN_GET_URL: &str = "https://account.bilibili.com/site/getCoin";

pub(super) const USER_CARD_GET_URL: &str = "https://api.bilibili.com/x/web-interface/card";

pub(super) const NICK_CHECK_URL: &str = "https://passport.bilibili.com/web/generic/check/nickname";

pub(super) const FANS_GET_URL: &str = "https://api.bilibili.com/x/relation/followers";

// region ## =================== Space ========================

pub(super) const MY_SPACE_GET_URL: &str = "https://api.bilibili.com/x/space/myinfo";

pub(super) const USER_SPACE_GET_URL: &str = "https://api.bilibili.com/x/space/acc/info";

pub(super) const PINNED_VIDEO_GET_URL: &str = "https://api.bilibili.com/x/space/top/arc";

pub(super) const MASTERPIECE_VIDEO_GET_URL: &str = "https://api.bilibili.com/x/space/masterpiece";

pub(super) const USER_TAGS_GET_URL: &str = "https://api.bilibili.com/x/space/acc/tags";

pub(super) const USER_SPACE_ANNOUNCEMENT_GET_URL: &str = "https://api.bilibili.com/x/space/notice";

pub(super) const USER_SPACE_SETTING_GET_URL: &str =
  "https://space.bilibili.com/ajax/settings/getSettings";

pub(super) const USER_RECENT_PLAYED_GAME_GET_URL: &str =
  "https://api.bilibili.com/x/space/lastplaygame";

pub(super) const USER_RECENT_COINED_VIDEO_GET_URL: &str =
  "https://api.bilibili.com/x/space/coin/video";

pub(super) const SPACE_VIDEO_GET_URL: &str = "https://api.bilibili.com/x/space/arc/search";

pub(super) const SPACE_ALBUM_INDEX_URL: &str = "https://api.bilibili.com/x/space/album/index";

pub(super) const SPACE_ALBUM_LIST_URL: &str =
  "https://api.vc.bilibili.com/link_draw/v1/doc/doc_list";

pub(super) const SPACE_CHANNEL_LIST_URL: &str = "https://api.bilibili.com/x/space/channel/list";

pub(super) const SPACE_CHANNEL_ARCHIVES_URL: &str =
  "https://api.bilibili.com/x/space/channel/video";

pub(super) const SPACE_COLLECTION_LIST_GET_URL: &str =
  "https://api.bilibili.com/x/v3/fav/folder/created/list-all";

pub(super) const SPACE_FAV_COLLECTION_LIST_GET_URL: &str =
  "https://api.bilibili.com/x/v3/fav/folder/collected/list";

pub(super) const SPACE_SUB_GET_URL: &str = "https://api.bilibili.com/x/space/bangumi/follow/list";

pub(super) const SPACE_SUB_TAGS_GET_URL: &str = "https://space.bilibili.com/ajax/tags/getSubList";

// endregion

// region ## =================== Relation ========================

pub(super) const FOLLOWING_GET_URL: &str = "https://api.bilibili.com/x/relation/followings";

pub(super) const FOLLOWING_SEARCH_URL: &str =
  "https://api.bilibili.com/x/relation/followings/search";

pub(super) const CO_FOLLOWING_GET_URL: &str = "https://api.bilibili.com/x/relation/same/followings";

pub(super) const QUIETLY_FOLLOWING_GET_URL: &str = "https://api.bilibili.com/x/relation/whispers";

pub(super) const BLACKLIST_GET_URL: &str = "https://api.bilibili.com/x/relation/blacks";

pub(super) const MODIFY_RELATION_URL: &str = "https://api.bilibili.com/x/relation/modify";

pub(super) const BATCH_MODIFY_RELATION_URL: &str =
  "https://api.bilibili.com/x/relation/batch/modify";

pub(super) const RELATION_QUERY_URL: &str = "https://api.bilibili.com/x/relation";

pub(super) const RELATION_BATCH_QUERY_URL: &str = "https://api.bilibili.com/x/relation/relations";

pub(super) const RELATION_QUERY_MUTUALLY: &str = "https://api.bilibili.com/x/space/acc/relation";

pub(super) const RELATION_QUERY_SPECIAL: &str = "https://api.bilibili.com/x/relation/tag/special";

// endregion

// region ## =================== Personal Centre ========================

pub(super) const ACCOUNT_INFO_GET_URL: &str = "https://api.bilibili.com/x/member/web/account";

pub(super) const EXP_REWARD_GET_URL: &str = "https://api.bilibili.com/x/member/web/exp/reward";

pub(super) const COIN_EXP_GET_URL: &str = "https://www.bilibili.com/plus/account/exp.php";

pub(super) const VIP_STAT_GET_URL: &str = "https://api.bilibili.com/x/vip/web/user/info";

pub(super) const SECURE_INFO_GET_URL: &str = "https://passport.bilibili.com/web/site/user/info";

pub(super) const REAL_NAME_INFO_GET_URL: &str = "https://api.bilibili.com/x/member/realname/status";

pub(super) const REAL_NAME_DETAILED_GET_URL: &str =
  "https://api.bilibili.com/x/member/realname/apply/status";

pub(super) const COIN_LOG_GET_URL: &str = "https://api.bilibili.com/x/member/web/coin/log";

pub(super) const CHANGE_BIO_URL: &str = "https://api.bilibili.com/x/member/web/sign/update";

// endregion

// region ## =================== Fav Folder ========================

pub(super) const FAVORITES_INFO_GET_URL: &str = "https://api.bilibili.com/x/v3/fav/folder/info";

pub(super) const FAVORITES_TYPES_GET_URL: &str =
  "https://api.bilibili.com/x/v3/fav/resource/partition";

pub(super) const FAVORITES_MEDIA_GET_URL: &str = "https://api.bilibili.com/x/v3/fav/resource/list";

// endregion

// endregion

// region =================== Album ========================

pub(super) const ALBUM_INFO_GET_URL: &str = "https://api.vc.bilibili.com/link_draw/v1/doc/detail";

pub(super) const ALBUM_UPLOAD_URL: &str = "https://api.vc.bilibili.com/api/v1/drawImage/upload";

// endregion

// region =================== Feed ========================

pub(super) const FEED_CONTENT_GET_URL: &str =
  "https://api.vc.bilibili.com/dynamic_svr/v1/dynamic_svr/get_dynamic_detail";

pub(super) const FEED_NEW_GET_URL: &str =
  "https://api.vc.bilibili.com/dynamic_svr/v1/dynamic_svr/dynamic_new";

pub(super) const FEED_HISTORY_GET_URL: &str =
  "https://api.vc.bilibili.com/dynamic_svr/v1/dynamic_svr/dynamic_history";

pub(super) const FEED_SPACE_GET_URL: &str =
  "https://api.vc.bilibili.com/dynamic_svr/v1/dynamic_svr/space_history";

pub(super) const FEED_LIVING_GET_URL: &str =
  "https://api.live.bilibili.com/xlive/web-ucenter/v1/xfetter/FeedList";

pub(super) const FEED_UPDATED_GET_URL: &str =
  "https://api.vc.bilibili.com/dynamic_svr/v1/dynamic_svr/w_dyn_uplist";

// endregion

// region =================== Live ========================

pub(super) const LIVE_INIT_INFO_GET_URL: &str =
  "https://api.live.bilibili.com/room/v1/Room/room_init";

pub(super) const LIVE_ROOM_INFO_URL: &str = "https://api.live.bilibili.com/room/v1/Room/get_info";

pub(super) const LIVE_AREA_URL: &str = "https://api.live.bilibili.com/room/v1/Area/getList";

pub(super) const LIVE_UID_TO_ROOM_ID: &str =
  "https://api.live.bilibili.com/room/v2/Room/room_id_by_uid";

pub(super) const LIVER_INFO_GET_URL: &str =
  "https://api.live.bilibili.com/live_user/v1/Master/info";

pub(super) const LIVE_SHOW_LIST_GET: &str =
  "https://api.live.bilibili.com/room/v1/Index/getShowList";

pub(super) const LIVE_CHECK_PWD_URL: &str =
  "https://api.live.bilibili.com/room/v1/Room/verify_room_pwd";

pub(super) const LIVE_HOVER_GET_URL: &str =
  "https://api.live.bilibili.com/room/v2/Index/getHoverInfo";

pub(super) const LIVE_DANMAKU_INFO_URL: &str =
  "https://api.live.bilibili.com/xlive/web-room/v1/index/getDanmuInfo";

pub(super) const LIVE_STREAM_FETCH_URL: &str =
  "https://api.live.bilibili.com/xlive/web-room/v2/index/getRoomPlayInfo";

pub(super) const LIVE_SIGN_URL: &str =
  "https://api.live.bilibili.com/xlive/web-ucenter/v1/sign/DoSign";

pub(super) const LIVE_SIGN_INFO_URL: &str =
  "https://api.live.bilibili.com/xlive/web-ucenter/v1/sign/WebGetSignInfo";

pub(super) const LIVE_SIGN_LAST_MONTH_URL: &str =
  "https://api.live.bilibili.com/sign/getLastMonthSignDays";

pub(super) const LIVE_RANKING_GET_URL: &str =
  "https://api.live.bilibili.com/rankdb/v1/Rank2018/getWebTop";

pub(super) const LIVE_MEDAL_RANK_GET_URL: &str =
  "https://api.live.bilibili.com/xlive/general-interface/v1/Rank/GetTotalMedalLevelRank";

pub(super) const LIVE_GUARD_LIST_GET_URL: &str =
  "https://api.live.bilibili.com/xlive/app-room/v2/guardTab/topList";

// endregion

// region =================== Passport ========================

pub(super) const QUERY_CAPTCHA_URL: &str = "https://passport.bilibili.com/x/passport-login/captcha";

/** params("act","getKey") */
pub(super) const RSA_GET_WEB_URL: &str = "https://passport.bilibili.com/login";

pub(super) const RSA_GET_APP_URL: &str = "https://passport.bilibili.com/api/oauth2/getKey";

pub(super) const LOGIN_WEB_URL: &str = "https://passport.bilibili.com/x/passport-login/web/login";

pub(super) const LOGIN_QRCODE_GET_WEB_URL: &str =
  "https://passport.bilibili.com/qrcode/getLoginUrl";

pub(super) const LOGIN_WEB_QRCODE_URL: &str = "https://passport.bilibili.com/qrcode/getLoginInfo";

pub(super) const GET_CALLING_CODE_URL: &str =
  "https://passport.bilibili.com/web/generic/country/list";

pub(super) const SEND_SMS_URL: &str = "https://passport.bilibili.com/x/passport-login/web/sms/send";

pub(super) const LOGIN_WEB_SMS_URL: &str =
  "https://passport.bilibili.com/x/passport-login/web/login/sms";

pub(super) const LOG_OUT_URL: &str = "https://passport.bilibili.com/login/exit/v2";

// endregion

// region =================== Message ========================

pub(super) const UNREAD_MESSAGE_COUNT_GET_URL: &str = "https://api.bilibili.com/x/msgfeed/unread";

pub(super) const UNREAD_WHISPER_COUNT_GET_URL: &str =
  "https://api.vc.bilibili.com/session_svr/v1/session_svr/single_unread";

pub(super) const SEND_MESSAGE_URL: &str = "https://api.vc.bilibili.com/web_im/v1/web_im/send_msg";

pub(super) const FETCH_MESSAGE_SESSIONS_URL: &str =
  "https://api.vc.bilibili.com/session_svr/v1/session_svr/get_sessions";

pub(super) const FETCH_NEW_MESSAGE_SESSIONS_URL: &str =
  "https://api.vc.bilibili.com/session_svr/v1/session_svr/new_sessions";

pub(super) const MESSAGE_SETTINGS_URL: &str =
  "https://api.vc.bilibili.com/link_setting/v1/link_setting/set";

pub(super) const FETCH_SESSION_MESSAGES_URL: &str =
  "https://api.vc.bilibili.com/svr_sync/v1/svr_sync/fetch_session_msgs";

// endregion

// region =================== Article ========================

pub(super) const ARTICLE_PAGE_URL: &str = "https://www.bilibili.com/read";

pub(super) const ARTICLE_BASIC_INFO_GET_URL: &str = "https://api.bilibili.com/x/article/viewinfo";

pub(super) const ARTICLE_SET_INFO_GET_URL: &str =
  "https://api.bilibili.com/x/article/list/web/articles";

// endregion

// region =================== History ========================

pub(super) const HISTORY_GET_URL: &str = "https://api.bilibili.com/x/web-interface/history/cursor";

pub(super) const HISTORY_DELETE_URL: &str = "https://api.bilibili.com/x/v2/history/delete";

pub(super) const HISTORY_CLEAR_URL: &str = "https://api.bilibili.com/x/v2/history/clear";

pub(super) const HISTORY_STOP_URL: &str = "https://api.bilibili.com/x/v2/history/shadow/set";

pub(super) const HISTORY_STATUS_QUERY_URL: &str = "https://api.bilibili.com/x/v2/history/shadow";

pub(super) const LATER_WATCH_ADD_URL: &str = "https://api.bilibili.com/x/v2/history/toview/add";

pub(super) const LATER_WATCH_ADD_CHANNEL_URL: &str =
  "https://space.bilibili.com/ajax/channel/addAllToView";

pub(super) const LATER_WATCH_GET_URL: &str = "https://api.bilibili.com/x/v2/history/toview";

pub(super) const LATER_WATCH_DELETE_URL: &str = "https://api.bilibili.com/x/v2/history/toview/del";

pub(super) const LATER_WATCH_CLEAR_URL: &str = "https://api.bilibili.com/x/v2/history/toview/clear";

// endregion

// region =================== Search ========================

pub(super) const SEARCH_ALL_URL: &str = "https://api.bilibili.com/x/web-interface/search/all/v2";

pub(super) const SEARCH_BY_TYPE_URL: &str = "https://api.bilibili.com/x/web-interface/search/type";

pub(super) const SEARCH_PLACEHOLDER_GET_URL: &str =
  "https://api.bilibili.com/x/web-interface/search/default";

pub(super) const SEARCH_RANKING_GET_URL: &str = "https://s.search.bilibili.com/main/hotword";

// endregion

// region =================== Comment ========================

pub(super) const COMMENT_GET_URL: &str = "https://api.bilibili.com/x/v2/reply";

pub(super) const COMMENT_LAZY_GET_URL: &str = "https://api.bilibili.com/x/v2/reply/main";

// endregion
