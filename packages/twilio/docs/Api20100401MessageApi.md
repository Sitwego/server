# \Api20100401MessageApi

All URIs are relative to *https://api.twilio.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_message**](Api20100401MessageApi.md#create_message) | **POST** /2010-04-01/Accounts/{AccountSid}/Messages.json | Send a message
[**delete_message**](Api20100401MessageApi.md#delete_message) | **DELETE** /2010-04-01/Accounts/{AccountSid}/Messages/{Sid}.json | Deletes a Message resource from your account
[**fetch_message**](Api20100401MessageApi.md#fetch_message) | **GET** /2010-04-01/Accounts/{AccountSid}/Messages/{Sid}.json | Fetch a specific Message
[**list_message**](Api20100401MessageApi.md#list_message) | **GET** /2010-04-01/Accounts/{AccountSid}/Messages.json | Retrieve a list of Message resources associated with a Twilio Account
[**update_message**](Api20100401MessageApi.md#update_message) | **POST** /2010-04-01/Accounts/{AccountSid}/Messages/{Sid}.json | Update a Message resource (used to redact Message `body` text and to cancel not-yet-sent messages)



## create_message

> models::ApiV2010AccountMessage create_message(account_sid, to, status_callback, application_sid, max_price, provide_feedback, attempt, validity_period, force_delivery, content_retention, address_retention, smart_encoded, persistent_action, traffic_type, shorten_urls, schedule_type, send_at, send_as_mms, content_variables, risk_check, from, messaging_service_sid, body, media_url, content_sid)
Send a message

Send a message

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) creating the Message resource. | [required] |
**to** | **String** | The recipient's phone number in [E.164](https://www.twilio.com/docs/glossary/what-e164) format (for SMS/MMS) or [channel address](https://www.twilio.com/docs/messaging/channels), e.g. `whatsapp:+15552229999`. | [required] |
**status_callback** | Option<**String**> | The URL of the endpoint to which Twilio sends [Message status callback requests](https://www.twilio.com/docs/sms/api/message-resource#twilios-request-to-the-statuscallback-url). URL must contain a valid hostname and underscores are not allowed. If you include this parameter with the `messaging_service_sid`, Twilio uses this URL instead of the Status Callback URL of the [Messaging Service](https://www.twilio.com/docs/messaging/api/service-resource).  |  |
**application_sid** | Option<**String**> | The SID of the associated [TwiML Application](https://www.twilio.com/docs/usage/api/applications). [Message status callback requests](https://www.twilio.com/docs/sms/api/message-resource#twilios-request-to-the-statuscallback-url) are sent to the TwiML App's `message_status_callback` URL. Note that the `status_callback` parameter of a request takes priority over the `application_sid` parameter; if both are included `application_sid` is ignored. |  |
**max_price** | Option<**f64**> | [OBSOLETE] This parameter will no longer have any effect as of 2024-06-03. |  |
**provide_feedback** | Option<**bool**> | Boolean indicating whether or not you intend to provide delivery confirmation feedback to Twilio (used in conjunction with the [Message Feedback subresource](https://www.twilio.com/docs/sms/api/message-feedback-resource)). Default value is `false`. |  |
**attempt** | Option<**i32**> | Total number of attempts made (including this request) to send the message regardless of the provider used |  |
**validity_period** | Option<**i32**> | The maximum length in seconds that the Message can remain in Twilio's outgoing message queue. If a queued Message exceeds the `validity_period`, the Message is not sent. Accepted values are integers from `1` to `36000`. Default value is `36000`. A `validity_period` greater than `5` is recommended. [Learn more about the validity period](https://www.twilio.com/blog/take-more-control-of-outbound-messages-using-validity-period-html) |  |
**force_delivery** | Option<**bool**> | Reserved |  |
**content_retention** | Option<[**models::MessageEnumContentRetention**](MessageEnumContentRetention.md)> |  |  |
**address_retention** | Option<[**models::MessageEnumAddressRetention**](MessageEnumAddressRetention.md)> |  |  |
**smart_encoded** | Option<**bool**> | Whether to detect Unicode characters that have a similar GSM-7 character and replace them. Can be: `true` or `false`. |  |
**persistent_action** | Option<[**Vec<String>**](String.md)> | Rich actions for non-SMS/MMS channels. Used for [sending location in WhatsApp messages](https://www.twilio.com/docs/whatsapp/message-features#location-messages-with-whatsapp). |  |
**traffic_type** | Option<[**models::MessageEnumTrafficType**](MessageEnumTrafficType.md)> |  |  |
**shorten_urls** | Option<**bool**> | For Messaging Services with [Link Shortening configured](https://www.twilio.com/docs/messaging/features/link-shortening) only: A Boolean indicating whether or not Twilio should shorten links in the `body` of the Message. Default value is `false`. If `true`, the `messaging_service_sid` parameter must also be provided. |  |
**schedule_type** | Option<[**models::MessageEnumScheduleType**](MessageEnumScheduleType.md)> |  |  |
**send_at** | Option<**String**> | The time that Twilio will send the message. Must be in ISO 8601 format. |  |
**send_as_mms** | Option<**bool**> | If set to `true`, Twilio delivers the message as a single MMS message, regardless of the presence of media. |  |
**content_variables** | Option<**String**> | For [Content Editor/API](https://www.twilio.com/docs/content) only: Key-value pairs of [Template variables](https://www.twilio.com/docs/content/using-variables-with-content-api) and their substitution values. `content_sid` parameter must also be provided. If values are not defined in the `content_variables` parameter, the [Template's default placeholder values](https://www.twilio.com/docs/content/content-api-resources#create-templates) are used. |  |
**risk_check** | Option<[**models::MessageEnumRiskCheck**](MessageEnumRiskCheck.md)> |  |  |
**from** | Option<**String**> | The sender's Twilio phone number (in [E.164](https://en.wikipedia.org/wiki/E.164) format), [alphanumeric sender ID](https://www.twilio.com/docs/sms/quickstart), [Wireless SIM](https://www.twilio.com/docs/iot/wireless/programmable-wireless-send-machine-machine-sms-commands), [short code](https://www.twilio.com/en-us/messaging/channels/sms/short-codes), or [channel address](https://www.twilio.com/docs/messaging/channels) (e.g., `whatsapp:+15554449999`). The value of the `from` parameter must be a sender that is hosted within Twilio and belongs to the Account creating the Message. If you are using `messaging_service_sid`, this parameter can be empty (Twilio assigns a `from` value from the Messaging Service's Sender Pool) or you can provide a specific sender from your Sender Pool. |  |
**messaging_service_sid** | Option<**String**> | The SID of the [Messaging Service](https://www.twilio.com/docs/messaging/services) you want to associate with the Message. When this parameter is provided and the `from` parameter is omitted, Twilio selects the optimal sender from the Messaging Service's Sender Pool. You may also provide a `from` parameter if you want to use a specific Sender from the Sender Pool. |  |
**body** | Option<**String**> | The text content of the outgoing message. Can be up to 1,600 characters in length. SMS only: If the `body` contains more than 160 [GSM-7](https://www.twilio.com/docs/glossary/what-is-gsm-7-character-encoding) characters (or 70 [UCS-2](https://www.twilio.com/docs/glossary/what-is-ucs-2-character-encoding) characters), the message is segmented and charged accordingly. For long `body` text, consider using the [send_as_mms parameter](https://www.twilio.com/blog/mms-for-long-text-messages). |  |
**media_url** | Option<[**Vec<String>**](String.md)> | The URL of media to include in the Message content. `jpeg`, `jpg`, `gif`, and `png` file types are fully supported by Twilio and content is formatted for delivery on destination devices. The media size limit is 5 MB for supported file types (`jpeg`, `jpg`, `png`, `gif`) and 500 KB for [other types](https://www.twilio.com/docs/messaging/guides/accepted-mime-types) of accepted media. To send more than one image in the message, provide multiple `media_url` parameters in the POST request. You can include up to ten `media_url` parameters per message. [International](https://support.twilio.com/hc/en-us/articles/223179808-Sending-and-receiving-MMS-messages) and [carrier](https://support.twilio.com/hc/en-us/articles/223133707-Is-MMS-supported-for-all-carriers-in-US-and-Canada-) limits apply. |  |
**content_sid** | Option<**String**> | For [Content Editor/API](https://www.twilio.com/docs/content) only: The SID of the Content Template to be used with the Message, e.g., `HXXXXXXXXXXXXXXXXXXXXXXXXXXXXX`. If this parameter is not provided, a Content Template is not used. Find the SID in the Console on the Content Editor page. For Content API users, the SID is found in Twilio's response when [creating the Template](https://www.twilio.com/docs/content/content-api-resources#create-templates) or by [fetching your Templates](https://www.twilio.com/docs/content/content-api-resources#fetch-all-content-resources). |  |

### Return type

[**models::ApiV2010AccountMessage**](api.v2010.account.message.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: application/x-www-form-urlencoded
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_message

> delete_message(account_sid, sid)
Deletes a Message resource from your account

Deletes a Message resource from your account

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) associated with the Message resource | [required] |
**sid** | **String** | The SID of the Message resource you wish to delete | [required] |

### Return type

 (empty response body)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## fetch_message

> models::ApiV2010AccountMessage fetch_message(account_sid, sid)
Fetch a specific Message

Fetch a specific Message

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) associated with the Message resource | [required] |
**sid** | **String** | The SID of the Message resource to be fetched | [required] |

### Return type

[**models::ApiV2010AccountMessage**](api.v2010.account.message.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_message

> models::ListMessageResponse list_message(account_sid, to, from, date_sent, date_sent_less_than, date_sent_greater_than, page_size, page, page_token)
Retrieve a list of Message resources associated with a Twilio Account

Retrieve a list of Message resources associated with a Twilio Account

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) associated with the Message resources. | [required] |
**to** | Option<**String**> | Filter by recipient. For example: Set this parameter to `+15558881111` to retrieve a list of Message resources sent to `+15558881111`. |  |
**from** | Option<**String**> | Filter by sender. For example: Set this parameter to `+15552229999` to retrieve a list of Message resources sent by `+15552229999`. |  |
**date_sent** | Option<**String**> | Filter by Message `sent_date`. Accepts GMT dates in the following formats: `YYYY-MM-DD` (to find Messages with a specific `sent_date`), `<=YYYY-MM-DD` (to find Messages with `sent_date`s on and before a specific date), and `>=YYYY-MM-DD` (to find Messages with `sent_dates` on and after a specific date). |  |
**date_sent_less_than** | Option<**String**> | Filter by Message `sent_date`. Accepts GMT dates in the following formats: `YYYY-MM-DD` (to find Messages with a specific `sent_date`), `<=YYYY-MM-DD` (to find Messages with `sent_date`s on and before a specific date), and `>=YYYY-MM-DD` (to find Messages with `sent_dates` on and after a specific date). |  |
**date_sent_greater_than** | Option<**String**> | Filter by Message `sent_date`. Accepts GMT dates in the following formats: `YYYY-MM-DD` (to find Messages with a specific `sent_date`), `<=YYYY-MM-DD` (to find Messages with `sent_date`s on and before a specific date), and `>=YYYY-MM-DD` (to find Messages with `sent_dates` on and after a specific date). |  |
**page_size** | Option<**i64**> | How many resources to return in each list page. The default is 50, and the maximum is 1000. |  |
**page** | Option<**i32**> | The page index. This value is simply for client state. |  |
**page_token** | Option<**String**> | The page token. This is provided by the API. |  |

### Return type

[**models::ListMessageResponse**](ListMessageResponse.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_message

> models::ApiV2010AccountMessage update_message(account_sid, sid, body, status)
Update a Message resource (used to redact Message `body` text and to cancel not-yet-sent messages)

Update a Message resource (used to redact Message `body` text and to cancel not-yet-sent messages)

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created the Message resources to update. | [required] |
**sid** | **String** | The SID of the Message resource to be updated | [required] |
**body** | Option<**String**> | The new `body` of the Message resource. To redact the text content of a Message, this parameter's value must be an empty string |  |
**status** | Option<[**models::MessageEnumUpdateStatus**](MessageEnumUpdateStatus.md)> |  |  |

### Return type

[**models::ApiV2010AccountMessage**](api.v2010.account.message.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: application/x-www-form-urlencoded
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

