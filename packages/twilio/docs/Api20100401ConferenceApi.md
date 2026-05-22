# \Api20100401ConferenceApi

All URIs are relative to *https://api.twilio.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**fetch_conference**](Api20100401ConferenceApi.md#fetch_conference) | **GET** /2010-04-01/Accounts/{AccountSid}/Conferences/{Sid}.json | Fetch an instance of a conference
[**list_conference**](Api20100401ConferenceApi.md#list_conference) | **GET** /2010-04-01/Accounts/{AccountSid}/Conferences.json | Retrieve a list of conferences belonging to the account used to make the request
[**update_conference**](Api20100401ConferenceApi.md#update_conference) | **POST** /2010-04-01/Accounts/{AccountSid}/Conferences/{Sid}.json | 



## fetch_conference

> models::ApiV2010AccountConference fetch_conference(account_sid, sid)
Fetch an instance of a conference

Fetch an instance of a conference

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created the Conference resource(s) to fetch. | [required] |
**sid** | **String** | The Twilio-provided string that uniquely identifies the Conference resource to fetch | [required] |

### Return type

[**models::ApiV2010AccountConference**](api.v2010.account.conference.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_conference

> models::ListConferenceResponse list_conference(account_sid, date_created, date_created_less_than, date_created_greater_than, date_updated, date_updated_less_than, date_updated_greater_than, friendly_name, status, page_size, page, page_token)
Retrieve a list of conferences belonging to the account used to make the request

Retrieve a list of conferences belonging to the account used to make the request

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created the Conference resource(s) to read. | [required] |
**date_created** | Option<**String**> | Only include conferences that were created on this date. Specify a date as `YYYY-MM-DD` in UTC, for example: `2009-07-06`, to read only conferences that were created on this date. You can also specify an inequality, such as `DateCreated<=YYYY-MM-DD`, to read conferences that were created on or before midnight of this date, and `DateCreated>=YYYY-MM-DD` to read conferences that were created on or after midnight of this date. |  |
**date_created_less_than** | Option<**String**> | Only include conferences that were created on this date. Specify a date as `YYYY-MM-DD` in UTC, for example: `2009-07-06`, to read only conferences that were created on this date. You can also specify an inequality, such as `DateCreated<=YYYY-MM-DD`, to read conferences that were created on or before midnight of this date, and `DateCreated>=YYYY-MM-DD` to read conferences that were created on or after midnight of this date. |  |
**date_created_greater_than** | Option<**String**> | Only include conferences that were created on this date. Specify a date as `YYYY-MM-DD` in UTC, for example: `2009-07-06`, to read only conferences that were created on this date. You can also specify an inequality, such as `DateCreated<=YYYY-MM-DD`, to read conferences that were created on or before midnight of this date, and `DateCreated>=YYYY-MM-DD` to read conferences that were created on or after midnight of this date. |  |
**date_updated** | Option<**String**> | Only include conferences that were last updated on this date. Specify a date as `YYYY-MM-DD` in UTC, for example: `2009-07-06`, to read only conferences that were last updated on this date. You can also specify an inequality, such as `DateUpdated<=YYYY-MM-DD`, to read conferences that were last updated on or before midnight of this date, and `DateUpdated>=YYYY-MM-DD` to read conferences that were last updated on or after midnight of this date. |  |
**date_updated_less_than** | Option<**String**> | Only include conferences that were last updated on this date. Specify a date as `YYYY-MM-DD` in UTC, for example: `2009-07-06`, to read only conferences that were last updated on this date. You can also specify an inequality, such as `DateUpdated<=YYYY-MM-DD`, to read conferences that were last updated on or before midnight of this date, and `DateUpdated>=YYYY-MM-DD` to read conferences that were last updated on or after midnight of this date. |  |
**date_updated_greater_than** | Option<**String**> | Only include conferences that were last updated on this date. Specify a date as `YYYY-MM-DD` in UTC, for example: `2009-07-06`, to read only conferences that were last updated on this date. You can also specify an inequality, such as `DateUpdated<=YYYY-MM-DD`, to read conferences that were last updated on or before midnight of this date, and `DateUpdated>=YYYY-MM-DD` to read conferences that were last updated on or after midnight of this date. |  |
**friendly_name** | Option<**String**> | The string that identifies the Conference resources to read. |  |
**status** | Option<[**ConferenceEnumStatus**](ConferenceEnumStatus.md)> | The status of the resources to read. Can be: `init`, `in-progress`, or `completed`. |  |
**page_size** | Option<**i64**> | How many resources to return in each list page. The default is 50, and the maximum is 1000. |  |
**page** | Option<**i32**> | The page index. This value is simply for client state. |  |
**page_token** | Option<**String**> | The page token. This is provided by the API. |  |

### Return type

[**models::ListConferenceResponse**](ListConferenceResponse.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_conference

> models::ApiV2010AccountConference update_conference(account_sid, sid, status, announce_url, announce_method)




### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created the Conference resource(s) to update. | [required] |
**sid** | **String** | The Twilio-provided string that uniquely identifies the Conference resource to update | [required] |
**status** | Option<[**models::ConferenceEnumUpdateStatus**](ConferenceEnumUpdateStatus.md)> |  |  |
**announce_url** | Option<**String**> | The URL we should call to announce something into the conference. The URL may return an MP3 file, a WAV file, or a TwiML document that contains `<Play>`, `<Say>`, `<Pause>`, or `<Redirect>` verbs. |  |
**announce_method** | Option<**String**> | The HTTP method used to call `announce_url`. Can be: `GET` or `POST` and the default is `POST` |  |

### Return type

[**models::ApiV2010AccountConference**](api.v2010.account.conference.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: application/x-www-form-urlencoded
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

