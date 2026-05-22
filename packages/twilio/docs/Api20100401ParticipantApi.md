# \Api20100401ParticipantApi

All URIs are relative to *https://api.twilio.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_participant**](Api20100401ParticipantApi.md#create_participant) | **POST** /2010-04-01/Accounts/{AccountSid}/Conferences/{ConferenceSid}/Participants.json | 
[**delete_participant**](Api20100401ParticipantApi.md#delete_participant) | **DELETE** /2010-04-01/Accounts/{AccountSid}/Conferences/{ConferenceSid}/Participants/{CallSid}.json | Kick a participant from a given conference
[**fetch_participant**](Api20100401ParticipantApi.md#fetch_participant) | **GET** /2010-04-01/Accounts/{AccountSid}/Conferences/{ConferenceSid}/Participants/{CallSid}.json | Fetch an instance of a participant
[**list_participant**](Api20100401ParticipantApi.md#list_participant) | **GET** /2010-04-01/Accounts/{AccountSid}/Conferences/{ConferenceSid}/Participants.json | Retrieve a list of participants belonging to the account used to make the request
[**update_participant**](Api20100401ParticipantApi.md#update_participant) | **POST** /2010-04-01/Accounts/{AccountSid}/Conferences/{ConferenceSid}/Participants/{CallSid}.json | Update the properties of the participant



## create_participant

> models::ApiV2010AccountConferenceParticipant create_participant(account_sid, conference_sid, from, to, status_callback, status_callback_method, status_callback_event, label, timeout, record, muted, beep, start_conference_on_enter, end_conference_on_exit, wait_url, wait_method, early_media, max_participants, conference_record, conference_trim, conference_status_callback, conference_status_callback_method, conference_status_callback_event, recording_channels, recording_status_callback, recording_status_callback_method, sip_auth_username, sip_auth_password, region, conference_recording_status_callback, conference_recording_status_callback_method, recording_status_callback_event, conference_recording_status_callback_event, coaching, call_sid_to_coach, jitter_buffer_size, byoc, caller_id, call_reason, recording_track, time_limit, machine_detection, machine_detection_timeout, machine_detection_speech_threshold, machine_detection_speech_end_threshold, machine_detection_silence_timeout, amd_status_callback, amd_status_callback_method, trim, call_token, client_notification_url, caller_display_name)




### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that will create the resource. | [required] |
**conference_sid** | **String** | The SID of the participant's conference. | [required] |
**from** | **String** | The phone number, Client identifier, or username portion of SIP address that made this call. Phone numbers are in [E.164](https://www.twilio.com/docs/glossary/what-e164) format (e.g., +16175551212). Client identifiers are formatted `client:name`. If using a phone number, it must be a Twilio number or a Verified [outgoing caller id](https://www.twilio.com/docs/voice/api/outgoing-caller-ids) for your account. If the `to` parameter is a phone number, `from` must also be a phone number. If `to` is sip address, this value of `from` should be a username portion to be used to populate the P-Asserted-Identity header that is passed to the SIP endpoint. | [required] |
**to** | **String** | The phone number, SIP address, Client, TwiML App identifier that received this call. Phone numbers are in [E.164](https://www.twilio.com/docs/glossary/what-e164) format (e.g., +16175551212). SIP addresses are formatted as `sip:name@company.com`. Client identifiers are formatted `client:name`. TwiML App identifiers are formatted `app:<APP_SID>`. [Custom parameters](https://www.twilio.com/docs/voice/api/conference-participant-resource#custom-parameters) may also be specified. | [required] |
**status_callback** | Option<**String**> | The URL we should call using the `status_callback_method` to send status information to your application. |  |
**status_callback_method** | Option<**String**> | The HTTP method we should use to call `status_callback`. Can be: `GET` and `POST` and defaults to `POST`. |  |
**status_callback_event** | Option<[**Vec<String>**](String.md)> | The conference state changes that should generate a call to `status_callback`. Can be: `initiated`, `ringing`, `answered`, and `completed`. Separate multiple values with a space. The default value is `completed`. |  |
**label** | Option<**String**> | A label for this participant. If one is supplied, it may subsequently be used to fetch, update or delete the participant. |  |
**timeout** | Option<**i32**> | The number of seconds that we should allow the phone to ring before assuming there is no answer. Can be an integer between `5` and `600`, inclusive. The default value is `60`. We always add a 5-second timeout buffer to outgoing calls, so  value of 10 would result in an actual timeout that was closer to 15 seconds. |  |
**record** | Option<**bool**> | Whether to record the participant and their conferences, including the time between conferences. Can be `true` or `false` and the default is `false`. |  |
**muted** | Option<**bool**> | Whether the agent is muted in the conference. Can be `true` or `false` and the default is `false`. |  |
**beep** | Option<**String**> | Whether to play a notification beep to the conference when the participant joins. Can be: `true`, `false`, `onEnter`, or `onExit`. The default value is `true`. |  |
**start_conference_on_enter** | Option<**bool**> | Whether to start the conference when the participant joins, if it has not already started. Can be: `true` or `false` and the default is `true`. If `false` and the conference has not started, the participant is muted and hears background music until another participant starts the conference. |  |
**end_conference_on_exit** | Option<**bool**> | Whether to end the conference when the participant leaves. Can be: `true` or `false` and defaults to `false`. |  |
**wait_url** | Option<**String**> | The URL that Twilio calls using the `wait_method` before the conference has started. The URL may return an MP3 file, a WAV file, or a TwiML document. The default value is the URL of our standard hold music. If you do not want anything to play while waiting for the conference to start, specify an empty string by setting `wait_url` to `''`. For more details on the allowable verbs within the `waitUrl`, see the `waitUrl` attribute in the [<Conference> TwiML instruction](https://www.twilio.com/docs/voice/twiml/conference#attributes-waiturl). |  |
**wait_method** | Option<**String**> | The HTTP method we should use to call `wait_url`. Can be `GET` or `POST` and the default is `POST`. When using a static audio file, this should be `GET` so that we can cache the file. |  |
**early_media** | Option<**bool**> | Whether to allow an agent to hear the state of the outbound call, including ringing or disconnect messages. Can be: `true` or `false` and defaults to `true`. |  |
**max_participants** | Option<**i32**> | The maximum number of participants in the conference. Can be a positive integer from `2` to `250`. The default value is `250`. |  |
**conference_record** | Option<**String**> | Whether to record the conference the participant is joining. Can be: `true`, `false`, `record-from-start`, and `do-not-record`. The default value is `false`. |  |
**conference_trim** | Option<**String**> | Whether to trim leading and trailing silence from the conference recording. Can be: `trim-silence` or `do-not-trim` and defaults to `trim-silence`. |  |
**conference_status_callback** | Option<**String**> | The URL we should call using the `conference_status_callback_method` when the conference events in `conference_status_callback_event` occur. Only the value set by the first participant to join the conference is used. Subsequent `conference_status_callback` values are ignored. |  |
**conference_status_callback_method** | Option<**String**> | The HTTP method we should use to call `conference_status_callback`. Can be: `GET` or `POST` and defaults to `POST`. |  |
**conference_status_callback_event** | Option<[**Vec<String>**](String.md)> | The conference state changes that should generate a call to `conference_status_callback`. Can be: `start`, `end`, `join`, `leave`, `mute`, `hold`, `modify`, `speaker`, and `announcement`. Separate multiple values with a space. Defaults to `start end`. |  |
**recording_channels** | Option<**String**> | The recording channels for the final recording. Can be: `mono` or `dual` and the default is `mono`. |  |
**recording_status_callback** | Option<**String**> | The URL that we should call using the `recording_status_callback_method` when the recording status changes. |  |
**recording_status_callback_method** | Option<**String**> | The HTTP method we should use when we call `recording_status_callback`. Can be: `GET` or `POST` and defaults to `POST`. |  |
**sip_auth_username** | Option<**String**> | The SIP username used for authentication. |  |
**sip_auth_password** | Option<**String**> | The SIP password for authentication. |  |
**region** | Option<**String**> | The [region](https://support.twilio.com/hc/en-us/articles/223132167-How-global-low-latency-routing-and-region-selection-work-for-conferences-and-Client-calls) where we should mix the recorded audio. Can be:`us1`, `us2`, `ie1`, `de1`, `sg1`, `br1`, `au1`, or `jp1`. |  |
**conference_recording_status_callback** | Option<**String**> | The URL we should call using the `conference_recording_status_callback_method` when the conference recording is available. |  |
**conference_recording_status_callback_method** | Option<**String**> | The HTTP method we should use to call `conference_recording_status_callback`. Can be: `GET` or `POST` and defaults to `POST`. |  |
**recording_status_callback_event** | Option<[**Vec<String>**](String.md)> | The recording state changes that should generate a call to `recording_status_callback`. Can be: `started`, `in-progress`, `paused`, `resumed`, `stopped`, `completed`, `failed`, and `absent`. Separate multiple values with a space, ex: `'in-progress completed failed'`. |  |
**conference_recording_status_callback_event** | Option<[**Vec<String>**](String.md)> | The conference recording state changes that generate a call to `conference_recording_status_callback`. Can be: `in-progress`, `completed`, `failed`, and `absent`. Separate multiple values with a space, ex: `'in-progress completed failed'` |  |
**coaching** | Option<**bool**> | Whether the participant is coaching another call. Can be: `true` or `false`. If not present, defaults to `false` unless `call_sid_to_coach` is defined. If `true`, `call_sid_to_coach` must be defined. |  |
**call_sid_to_coach** | Option<**String**> | The SID of the participant who is being `coached`. The participant being coached is the only participant who can hear the participant who is `coaching`. |  |
**jitter_buffer_size** | Option<**String**> | Jitter buffer size for the connecting participant. Twilio will use this setting to apply Jitter Buffer before participant's audio is mixed into the conference. Can be: `off`, `small`, `medium`, and `large`. Default to `large`. |  |
**byoc** | Option<**String**> | The SID of a BYOC (Bring Your Own Carrier) trunk to route this call with. Note that `byoc` is only meaningful when `to` is a phone number; it will otherwise be ignored. (Beta) |  |
**caller_id** | Option<**String**> | The phone number, Client identifier, or username portion of SIP address that made this call. Phone numbers are in [E.164](https://www.twilio.com/docs/glossary/what-e164) format (e.g., +16175551212). Client identifiers are formatted `client:name`. If using a phone number, it must be a Twilio number or a Verified [outgoing caller id](https://www.twilio.com/docs/voice/api/outgoing-caller-ids) for your account. If the `to` parameter is a phone number, `callerId` must also be a phone number. If `to` is sip address, this value of `callerId` should be a username portion to be used to populate the From header that is passed to the SIP endpoint. |  |
**call_reason** | Option<**String**> | The Reason for the outgoing call. Use it to specify the purpose of the call that is presented on the called party's phone. (Branded Calls Beta) |  |
**recording_track** | Option<**String**> | The audio track to record for the call. Can be: `inbound`, `outbound` or `both`. The default is `both`. `inbound` records the audio that is received by Twilio. `outbound` records the audio that is sent from Twilio. `both` records the audio that is received and sent by Twilio. |  |
**time_limit** | Option<**i32**> | The maximum duration of the call in seconds. Constraints depend on account and configuration. |  |
**machine_detection** | Option<**String**> | Whether to detect if a human, answering machine, or fax has picked up the call. Can be: `Enable` or `DetectMessageEnd`. Use `Enable` if you would like us to return `AnsweredBy` as soon as the called party is identified. Use `DetectMessageEnd`, if you would like to leave a message on an answering machine. For more information, see [Answering Machine Detection](https://www.twilio.com/docs/voice/answering-machine-detection). |  |
**machine_detection_timeout** | Option<**i32**> | The number of seconds that we should attempt to detect an answering machine before timing out and sending a voice request with `AnsweredBy` of `unknown`. The default timeout is 30 seconds. |  |
**machine_detection_speech_threshold** | Option<**i32**> | The number of milliseconds that is used as the measuring stick for the length of the speech activity, where durations lower than this value will be interpreted as a human and longer than this value as a machine. Possible Values: 1000-6000. Default: 2400. |  |
**machine_detection_speech_end_threshold** | Option<**i32**> | The number of milliseconds of silence after speech activity at which point the speech activity is considered complete. Possible Values: 500-5000. Default: 1200. |  |
**machine_detection_silence_timeout** | Option<**i32**> | The number of milliseconds of initial silence after which an `unknown` AnsweredBy result will be returned. Possible Values: 2000-10000. Default: 5000. |  |
**amd_status_callback** | Option<**String**> | The URL that we should call using the `amd_status_callback_method` to notify customer application whether the call was answered by human, machine or fax. |  |
**amd_status_callback_method** | Option<**String**> | The HTTP method we should use when calling the `amd_status_callback` URL. Can be: `GET` or `POST` and the default is `POST`. |  |
**trim** | Option<**String**> | Whether to trim any leading and trailing silence from the participant recording. Can be: `trim-silence` or `do-not-trim` and the default is `trim-silence`. |  |
**call_token** | Option<**String**> | A token string needed to invoke a forwarded call. A call_token is generated when an incoming call is received on a Twilio number. Pass an incoming call's call_token value to a forwarded call via the call_token parameter when creating a new call. A forwarded call should bear the same CallerID of the original incoming call. |  |
**client_notification_url** | Option<**String**> | The URL that we should use to deliver `push call notification`. |  |
**caller_display_name** | Option<**String**> | The name that populates the display name in the From header. Must be between 2 and 255 characters. Only applicable for calls to sip address. |  |

### Return type

[**models::ApiV2010AccountConferenceParticipant**](api.v2010.account.conference.participant.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: application/x-www-form-urlencoded
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_participant

> delete_participant(account_sid, conference_sid, call_sid)
Kick a participant from a given conference

Kick a participant from a given conference

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created the Participant resources to delete. | [required] |
**conference_sid** | **String** | The SID of the conference with the participants to delete. | [required] |
**call_sid** | **String** | The [Call](https://www.twilio.com/docs/voice/api/call-resource) SID or label of the participant to delete. Non URL safe characters in a label must be percent encoded, for example, a space character is represented as %20. | [required] |

### Return type

 (empty response body)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## fetch_participant

> models::ApiV2010AccountConferenceParticipant fetch_participant(account_sid, conference_sid, call_sid)
Fetch an instance of a participant

Fetch an instance of a participant

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created the Participant resource to fetch. | [required] |
**conference_sid** | **String** | The SID of the conference with the participant to fetch. | [required] |
**call_sid** | **String** | The [Call](https://www.twilio.com/docs/voice/api/call-resource) SID or label of the participant to fetch. Non URL safe characters in a label must be percent encoded, for example, a space character is represented as %20. | [required] |

### Return type

[**models::ApiV2010AccountConferenceParticipant**](api.v2010.account.conference.participant.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_participant

> models::ListParticipantResponse list_participant(account_sid, conference_sid, muted, hold, coaching, page_size, page, page_token)
Retrieve a list of participants belonging to the account used to make the request

Retrieve a list of participants belonging to the account used to make the request

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created the Participant resources to read. | [required] |
**conference_sid** | **String** | The SID of the conference with the participants to read. | [required] |
**muted** | Option<**bool**> | Whether to return only participants that are muted. Can be: `true` or `false`. |  |
**hold** | Option<**bool**> | Whether to return only participants that are on hold. Can be: `true` or `false`. |  |
**coaching** | Option<**bool**> | Whether to return only participants who are coaching another call. Can be: `true` or `false`. |  |
**page_size** | Option<**i64**> | How many resources to return in each list page. The default is 50, and the maximum is 1000. |  |
**page** | Option<**i32**> | The page index. This value is simply for client state. |  |
**page_token** | Option<**String**> | The page token. This is provided by the API. |  |

### Return type

[**models::ListParticipantResponse**](ListParticipantResponse.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_participant

> models::ApiV2010AccountConferenceParticipant update_participant(account_sid, conference_sid, call_sid, muted, hold, hold_url, hold_method, announce_url, announce_method, wait_url, wait_method, beep_on_exit, end_conference_on_exit, coaching, call_sid_to_coach)
Update the properties of the participant

Update the properties of the participant

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created the Participant resources to update. | [required] |
**conference_sid** | **String** | The SID of the conference with the participant to update. | [required] |
**call_sid** | **String** | The [Call](https://www.twilio.com/docs/voice/api/call-resource) SID or label of the participant to update. Non URL safe characters in a label must be percent encoded, for example, a space character is represented as %20. | [required] |
**muted** | Option<**bool**> | Whether the participant should be muted. Can be `true` or `false`. `true` will mute the participant, and `false` will un-mute them. Anything value other than `true` or `false` is interpreted as `false`. |  |
**hold** | Option<**bool**> | Whether the participant should be on hold. Can be: `true` or `false`. `true` puts the participant on hold, and `false` lets them rejoin the conference. |  |
**hold_url** | Option<**String**> | The URL we call using the `hold_method` for music that plays when the participant is on hold. The URL may return an MP3 file, a WAV file, or a TwiML document that contains `<Play>`, `<Say>`, `<Pause>`, or `<Redirect>` verbs. |  |
**hold_method** | Option<**String**> | The HTTP method we should use to call `hold_url`. Can be: `GET` or `POST` and the default is `GET`. |  |
**announce_url** | Option<**String**> | The URL we call using the `announce_method` for an announcement to the participant. The URL may return an MP3 file, a WAV file, or a TwiML document that contains `<Play>`, `<Say>`, `<Pause>`, or `<Redirect>` verbs. |  |
**announce_method** | Option<**String**> | The HTTP method we should use to call `announce_url`. Can be: `GET` or `POST` and defaults to `POST`. |  |
**wait_url** | Option<**String**> | The URL that Twilio calls using the `wait_method` before the conference has started. The URL may return an MP3 file, a WAV file, or a TwiML document. The default value is the URL of our standard hold music. If you do not want anything to play while waiting for the conference to start, specify an empty string by setting `wait_url` to `''`. For more details on the allowable verbs within the `waitUrl`, see the `waitUrl` attribute in the [<Conference> TwiML instruction](https://www.twilio.com/docs/voice/twiml/conference#attributes-waiturl). |  |
**wait_method** | Option<**String**> | The HTTP method we should use to call `wait_url`. Can be `GET` or `POST` and the default is `POST`. When using a static audio file, this should be `GET` so that we can cache the file. |  |
**beep_on_exit** | Option<**bool**> | Whether to play a notification beep to the conference when the participant exits. Can be: `true` or `false`. |  |
**end_conference_on_exit** | Option<**bool**> | Whether to end the conference when the participant leaves. Can be: `true` or `false` and defaults to `false`. |  |
**coaching** | Option<**bool**> | Whether the participant is coaching another call. Can be: `true` or `false`. If not present, defaults to `false` unless `call_sid_to_coach` is defined. If `true`, `call_sid_to_coach` must be defined. |  |
**call_sid_to_coach** | Option<**String**> | The SID of the participant who is being `coached`. The participant being coached is the only participant who can hear the participant who is `coaching`. |  |

### Return type

[**models::ApiV2010AccountConferenceParticipant**](api.v2010.account.conference.participant.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: application/x-www-form-urlencoded
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

