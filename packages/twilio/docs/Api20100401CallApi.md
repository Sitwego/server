# \Api20100401CallApi

All URIs are relative to *https://api.twilio.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_call**](Api20100401CallApi.md#create_call) | **POST** /2010-04-01/Accounts/{AccountSid}/Calls.json | Create a new outgoing call to phones, SIP-enabled endpoints or Twilio Client connections
[**delete_call**](Api20100401CallApi.md#delete_call) | **DELETE** /2010-04-01/Accounts/{AccountSid}/Calls/{Sid}.json | Delete a Call record from your account. Once the record is deleted, it will no longer appear in the API and Account Portal logs.
[**fetch_call**](Api20100401CallApi.md#fetch_call) | **GET** /2010-04-01/Accounts/{AccountSid}/Calls/{Sid}.json | Fetch the call specified by the provided Call SID
[**list_call**](Api20100401CallApi.md#list_call) | **GET** /2010-04-01/Accounts/{AccountSid}/Calls.json | Retrieves a collection of calls made to and from your account
[**update_call**](Api20100401CallApi.md#update_call) | **POST** /2010-04-01/Accounts/{AccountSid}/Calls/{Sid}.json | Initiates a call redirect or terminates a call



## create_call

> models::ApiV2010AccountCall create_call(account_sid, to, from, method, fallback_url, fallback_method, status_callback, status_callback_event, status_callback_method, send_digits, timeout, record, recording_channels, recording_status_callback, recording_status_callback_method, sip_auth_username, sip_auth_password, machine_detection, machine_detection_timeout, recording_status_callback_event, trim, caller_id, machine_detection_speech_threshold, machine_detection_speech_end_threshold, machine_detection_silence_timeout, async_amd, async_amd_status_callback, async_amd_status_callback_method, byoc, call_reason, call_token, recording_track, time_limit, client_notification_url, url, twiml, application_sid)
Create a new outgoing call to phones, SIP-enabled endpoints or Twilio Client connections

Create a new outgoing call to phones, SIP-enabled endpoints or Twilio Client connections

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that will create the resource. | [required] |
**to** | **String** | The phone number, SIP address, or client identifier to call. | [required] |
**from** | **String** | The phone number or client identifier to use as the caller id. If using a phone number, it must be a Twilio number or a Verified [outgoing caller id](https://www.twilio.com/docs/voice/api/outgoing-caller-ids) for your account. If the `to` parameter is a phone number, `From` must also be a phone number. | [required] |
**method** | Option<**String**> | The HTTP method we should use when calling the `url` parameter's value. Can be: `GET` or `POST` and the default is `POST`. If an `application_sid` parameter is present, this parameter is ignored. |  |
**fallback_url** | Option<**String**> | The URL that we call using the `fallback_method` if an error occurs when requesting or executing the TwiML at `url`. If an `application_sid` parameter is present, this parameter is ignored. |  |
**fallback_method** | Option<**String**> | The HTTP method that we should use to request the `fallback_url`. Can be: `GET` or `POST` and the default is `POST`. If an `application_sid` parameter is present, this parameter is ignored. |  |
**status_callback** | Option<**String**> | The URL we should call using the `status_callback_method` to send status information to your application. If no `status_callback_event` is specified, we will send the `completed` status. If an `application_sid` parameter is present, this parameter is ignored. URLs must contain a valid hostname (underscores are not permitted). |  |
**status_callback_event** | Option<[**Vec<String>**](String.md)> | The call progress events that we will send to the `status_callback` URL. Can be: `initiated`, `ringing`, `answered`, and `completed`. If no event is specified, we send the `completed` status. If you want to receive multiple events, specify each one in a separate `status_callback_event` parameter. See the code sample for [monitoring call progress](https://www.twilio.com/docs/voice/api/call-resource?code-sample=code-create-a-call-resource-and-specify-a-statuscallbackevent&code-sdk-version=json). If an `application_sid` is present, this parameter is ignored. |  |
**status_callback_method** | Option<**String**> | The HTTP method we should use when calling the `status_callback` URL. Can be: `GET` or `POST` and the default is `POST`. If an `application_sid` parameter is present, this parameter is ignored. |  |
**send_digits** | Option<**String**> | The string of keys to dial after connecting to the number, with a maximum length of 32 digits. Valid digits in the string include any digit (`0`-`9`), '`A`', '`B`', '`C`', '`D`', '`#`', and '`*`'. You can also use '`w`' to insert a half-second pause and '`W`' to insert a one-second pause. For example, to pause for one second after connecting and then dial extension 1234 followed by the # key, set this parameter to `W1234#`. Be sure to URL-encode this string because the '`#`' character has special meaning in a URL. If both `SendDigits` and `MachineDetection` parameters are provided, then `MachineDetection` will be ignored. |  |
**timeout** | Option<**i32**> | The integer number of seconds that we should allow the phone to ring before assuming there is no answer. The default is `60` seconds and the maximum is `600` seconds. For some call flows, we will add a 5-second buffer to the timeout value you provide. For this reason, a timeout value of 10 seconds could result in an actual timeout closer to 15 seconds. You can set this to a short time, such as `15` seconds, to hang up before reaching an answering machine or voicemail. |  |
**record** | Option<**bool**> | Whether to record the call. Can be `true` to record the phone call, or `false` to not. The default is `false`. The `recording_url` is sent to the `status_callback` URL. |  |
**recording_channels** | Option<**String**> | The number of channels in the final recording. Can be: `mono` or `dual`. The default is `mono`. `mono` records both legs of the call in a single channel of the recording file. `dual` records each leg to a separate channel of the recording file. The first channel of a dual-channel recording contains the parent call and the second channel contains the child call. |  |
**recording_status_callback** | Option<**String**> | The URL that we call when the recording is available to be accessed. |  |
**recording_status_callback_method** | Option<**String**> | The HTTP method we should use when calling the `recording_status_callback` URL. Can be: `GET` or `POST` and the default is `POST`. |  |
**sip_auth_username** | Option<**String**> | The username used to authenticate the caller making a SIP call. |  |
**sip_auth_password** | Option<**String**> | The password required to authenticate the user account specified in `sip_auth_username`. |  |
**machine_detection** | Option<**String**> | Whether to detect if a human, answering machine, or fax has picked up the call. Can be: `Enable` or `DetectMessageEnd`. Use `Enable` if you would like us to return `AnsweredBy` as soon as the called party is identified. Use `DetectMessageEnd`, if you would like to leave a message on an answering machine. If `send_digits` is provided, this parameter is ignored. For more information, see [Answering Machine Detection](https://www.twilio.com/docs/voice/answering-machine-detection). |  |
**machine_detection_timeout** | Option<**i32**> | The number of seconds that we should attempt to detect an answering machine before timing out and sending a voice request with `AnsweredBy` of `unknown`. The default timeout is 30 seconds. |  |
**recording_status_callback_event** | Option<[**Vec<String>**](String.md)> | The recording status events that will trigger calls to the URL specified in `recording_status_callback`. Can be: `in-progress`, `completed` and `absent`. Defaults to `completed`. Separate  multiple values with a space. |  |
**trim** | Option<**String**> | Whether to trim any leading and trailing silence from the recording. Can be: `trim-silence` or `do-not-trim` and the default is `trim-silence`. |  |
**caller_id** | Option<**String**> | The phone number, SIP address, or Client identifier that made this call. Phone numbers are in [E.164 format](https://wwnw.twilio.com/docs/glossary/what-e164) (e.g., +16175551212). SIP addresses are formatted as `name@company.com`. |  |
**machine_detection_speech_threshold** | Option<**i32**> | The number of milliseconds that is used as the measuring stick for the length of the speech activity, where durations lower than this value will be interpreted as a human and longer than this value as a machine. Possible Values: 1000-6000. Default: 2400. |  |
**machine_detection_speech_end_threshold** | Option<**i32**> | The number of milliseconds of silence after speech activity at which point the speech activity is considered complete. Possible Values: 500-5000. Default: 1200. |  |
**machine_detection_silence_timeout** | Option<**i32**> | The number of milliseconds of initial silence after which an `unknown` AnsweredBy result will be returned. Possible Values: 2000-10000. Default: 5000. |  |
**async_amd** | Option<**String**> | Select whether to perform answering machine detection in the background. Default, blocks the execution of the call until Answering Machine Detection is completed. Can be: `true` or `false`. |  |
**async_amd_status_callback** | Option<**String**> | The URL that we should call using the `async_amd_status_callback_method` to notify customer application whether the call was answered by human, machine or fax. |  |
**async_amd_status_callback_method** | Option<**String**> | The HTTP method we should use when calling the `async_amd_status_callback` URL. Can be: `GET` or `POST` and the default is `POST`. |  |
**byoc** | Option<**String**> | The SID of a BYOC (Bring Your Own Carrier) trunk to route this call with. Note that `byoc` is only meaningful when `to` is a phone number; it will otherwise be ignored. (Beta) |  |
**call_reason** | Option<**String**> | The Reason for the outgoing call. Use it to specify the purpose of the call that is presented on the called party's phone. (Branded Calls Beta) |  |
**call_token** | Option<**String**> | A token string needed to invoke a forwarded call. A call_token is generated when an incoming call is received on a Twilio number. Pass an incoming call's call_token value to a forwarded call via the call_token parameter when creating a new call. A forwarded call should bear the same CallerID of the original incoming call. |  |
**recording_track** | Option<**String**> | The audio track to record for the call. Can be: `inbound`, `outbound` or `both`. The default is `both`. `inbound` records the audio that is received by Twilio. `outbound` records the audio that is generated from Twilio. `both` records the audio that is received and generated by Twilio. |  |
**time_limit** | Option<**i32**> | The maximum duration of the call in seconds. Constraints depend on account and configuration. |  |
**client_notification_url** | Option<**String**> | The URL that we should use to deliver `push call notification`. |  |
**url** | Option<**String**> | The absolute URL that returns the TwiML instructions for the call. We will call this URL using the `method` when the call connects. For more information, see the [Url Parameter](https://www.twilio.com/docs/voice/make-calls#specify-a-url-parameter) section in [Making Calls](https://www.twilio.com/docs/voice/make-calls). |  |
**twiml** | Option<**String**> | TwiML instructions for the call Twilio will use without fetching Twiml from url parameter. If both `twiml` and `url` are provided then `twiml` parameter will be ignored. Max 4000 characters. |  |
**application_sid** | Option<**String**> | The SID of the Application resource that will handle the call, if the call will be handled by an application. |  |

### Return type

[**models::ApiV2010AccountCall**](api.v2010.account.call.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: application/x-www-form-urlencoded
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_call

> delete_call(account_sid, sid)
Delete a Call record from your account. Once the record is deleted, it will no longer appear in the API and Account Portal logs.

Delete a Call record from your account. Once the record is deleted, it will no longer appear in the API and Account Portal logs.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created the Call resource(s) to delete. | [required] |
**sid** | **String** | The Twilio-provided Call SID that uniquely identifies the Call resource to delete | [required] |

### Return type

 (empty response body)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## fetch_call

> models::ApiV2010AccountCall fetch_call(account_sid, sid)
Fetch the call specified by the provided Call SID

Fetch the call specified by the provided Call SID

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created the Call resource(s) to fetch. | [required] |
**sid** | **String** | The SID of the Call resource to fetch. | [required] |

### Return type

[**models::ApiV2010AccountCall**](api.v2010.account.call.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_call

> models::ListCallResponse list_call(account_sid, to, from, parent_call_sid, status, start_time, start_time_less_than, start_time_greater_than, end_time, end_time_less_than, end_time_greater_than, page_size, page, page_token)
Retrieves a collection of calls made to and from your account

Retrieves a collection of calls made to and from your account

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created the Call resource(s) to read. | [required] |
**to** | Option<**String**> | Only show calls made to this phone number, SIP address, Client identifier or SIM SID. |  |
**from** | Option<**String**> | Only include calls from this phone number, SIP address, Client identifier or SIM SID. |  |
**parent_call_sid** | Option<**String**> | Only include calls spawned by calls with this SID. |  |
**status** | Option<[**CallEnumStatus**](CallEnumStatus.md)> | The status of the calls to include. Can be: `queued`, `ringing`, `in-progress`, `canceled`, `completed`, `failed`, `busy`, or `no-answer`. |  |
**start_time** | Option<**String**> | Only include calls that started on this date. Specify a date as `YYYY-MM-DD` in UTC, for example: `2009-07-06`, to read only calls that started on this date. |  |
**start_time_less_than** | Option<**String**> | Only include calls that started before this date. Specify a date as `YYYY-MM-DD` in UTC, for example: `2009-07-06`, to read only calls that started before this date. |  |
**start_time_greater_than** | Option<**String**> | Only include calls that started on or after this date. Specify a date as `YYYY-MM-DD` in UTC, for example: `2009-07-06`, to read only calls that started on or after this date. |  |
**end_time** | Option<**String**> | Only include calls that ended on this date. Specify a date as `YYYY-MM-DD` in UTC, for example: `2009-07-06`, to read only calls that ended on this date. |  |
**end_time_less_than** | Option<**String**> | Only include calls that ended before this date. Specify a date as `YYYY-MM-DD` in UTC, for example: `2009-07-06`, to read only calls that ended before this date. |  |
**end_time_greater_than** | Option<**String**> | Only include calls that ended on or after this date. Specify a date as `YYYY-MM-DD` in UTC, for example: `2009-07-06`, to read only calls that ended on or after this date. |  |
**page_size** | Option<**i64**> | How many resources to return in each list page. The default is 50, and the maximum is 1000. |  |
**page** | Option<**i32**> | The page index. This value is simply for client state. |  |
**page_token** | Option<**String**> | The page token. This is provided by the API. |  |

### Return type

[**models::ListCallResponse**](ListCallResponse.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_call

> models::ApiV2010AccountCall update_call(account_sid, sid, url, method, status, fallback_url, fallback_method, status_callback, status_callback_method, twiml, time_limit)
Initiates a call redirect or terminates a call

Initiates a call redirect or terminates a call

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created the Call resource(s) to update. | [required] |
**sid** | **String** | The Twilio-provided string that uniquely identifies the Call resource to update | [required] |
**url** | Option<**String**> | The absolute URL that returns the TwiML instructions for the call. We will call this URL using the `method` when the call connects. For more information, see the [Url Parameter](https://www.twilio.com/docs/voice/make-calls#specify-a-url-parameter) section in [Making Calls](https://www.twilio.com/docs/voice/make-calls). |  |
**method** | Option<**String**> | The HTTP method we should use when calling the `url`. Can be: `GET` or `POST` and the default is `POST`. If an `application_sid` parameter is present, this parameter is ignored. |  |
**status** | Option<[**models::CallEnumUpdateStatus**](CallEnumUpdateStatus.md)> |  |  |
**fallback_url** | Option<**String**> | The URL that we call using the `fallback_method` if an error occurs when requesting or executing the TwiML at `url`. If an `application_sid` parameter is present, this parameter is ignored. |  |
**fallback_method** | Option<**String**> | The HTTP method that we should use to request the `fallback_url`. Can be: `GET` or `POST` and the default is `POST`. If an `application_sid` parameter is present, this parameter is ignored. |  |
**status_callback** | Option<**String**> | The URL we should call using the `status_callback_method` to send status information to your application. If no `status_callback_event` is specified, we will send the `completed` status. If an `application_sid` parameter is present, this parameter is ignored. URLs must contain a valid hostname (underscores are not permitted). |  |
**status_callback_method** | Option<**String**> | The HTTP method we should use when requesting the `status_callback` URL. Can be: `GET` or `POST` and the default is `POST`. If an `application_sid` parameter is present, this parameter is ignored. |  |
**twiml** | Option<**String**> | TwiML instructions for the call Twilio will use without fetching Twiml from url. Twiml and url parameters are mutually exclusive |  |
**time_limit** | Option<**i32**> | The maximum duration of the call in seconds. Constraints depend on account and configuration. |  |

### Return type

[**models::ApiV2010AccountCall**](api.v2010.account.call.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: application/x-www-form-urlencoded
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

