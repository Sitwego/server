# \Api20100401OutgoingCallerIdApi

All URIs are relative to *https://api.twilio.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**delete_outgoing_caller_id**](Api20100401OutgoingCallerIdApi.md#delete_outgoing_caller_id) | **DELETE** /2010-04-01/Accounts/{AccountSid}/OutgoingCallerIds/{Sid}.json | Delete the caller-id specified from the account
[**fetch_outgoing_caller_id**](Api20100401OutgoingCallerIdApi.md#fetch_outgoing_caller_id) | **GET** /2010-04-01/Accounts/{AccountSid}/OutgoingCallerIds/{Sid}.json | Fetch an outgoing-caller-id belonging to the account used to make the request
[**list_outgoing_caller_id**](Api20100401OutgoingCallerIdApi.md#list_outgoing_caller_id) | **GET** /2010-04-01/Accounts/{AccountSid}/OutgoingCallerIds.json | Retrieve a list of outgoing-caller-ids belonging to the account used to make the request
[**update_outgoing_caller_id**](Api20100401OutgoingCallerIdApi.md#update_outgoing_caller_id) | **POST** /2010-04-01/Accounts/{AccountSid}/OutgoingCallerIds/{Sid}.json | Updates the caller-id



## delete_outgoing_caller_id

> delete_outgoing_caller_id(account_sid, sid)
Delete the caller-id specified from the account

Delete the caller-id specified from the account

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created the OutgoingCallerId resources to delete. | [required] |
**sid** | **String** | The Twilio-provided string that uniquely identifies the OutgoingCallerId resource to delete. | [required] |

### Return type

 (empty response body)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## fetch_outgoing_caller_id

> models::ApiV2010AccountOutgoingCallerId fetch_outgoing_caller_id(account_sid, sid)
Fetch an outgoing-caller-id belonging to the account used to make the request

Fetch an outgoing-caller-id belonging to the account used to make the request

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created the OutgoingCallerId resource to fetch. | [required] |
**sid** | **String** | The Twilio-provided string that uniquely identifies the OutgoingCallerId resource to fetch. | [required] |

### Return type

[**models::ApiV2010AccountOutgoingCallerId**](api.v2010.account.outgoing_caller_id.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_outgoing_caller_id

> models::ListOutgoingCallerIdResponse list_outgoing_caller_id(account_sid, phone_number, friendly_name, page_size, page, page_token)
Retrieve a list of outgoing-caller-ids belonging to the account used to make the request

Retrieve a list of outgoing-caller-ids belonging to the account used to make the request

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created the OutgoingCallerId resources to read. | [required] |
**phone_number** | Option<**String**> | The phone number of the OutgoingCallerId resources to read. |  |
**friendly_name** | Option<**String**> | The string that identifies the OutgoingCallerId resources to read. |  |
**page_size** | Option<**i64**> | How many resources to return in each list page. The default is 50, and the maximum is 1000. |  |
**page** | Option<**i32**> | The page index. This value is simply for client state. |  |
**page_token** | Option<**String**> | The page token. This is provided by the API. |  |

### Return type

[**models::ListOutgoingCallerIdResponse**](ListOutgoingCallerIdResponse.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_outgoing_caller_id

> models::ApiV2010AccountOutgoingCallerId update_outgoing_caller_id(account_sid, sid, friendly_name)
Updates the caller-id

Updates the caller-id

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created the OutgoingCallerId resources to update. | [required] |
**sid** | **String** | The Twilio-provided string that uniquely identifies the OutgoingCallerId resource to update. | [required] |
**friendly_name** | Option<**String**> | A descriptive string that you create to describe the resource. It can be up to 64 characters long. |  |

### Return type

[**models::ApiV2010AccountOutgoingCallerId**](api.v2010.account.outgoing_caller_id.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: application/x-www-form-urlencoded
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

