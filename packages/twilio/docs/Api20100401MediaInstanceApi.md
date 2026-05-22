# \Api20100401MediaInstanceApi

All URIs are relative to *https://api.twilio.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**delete_media**](Api20100401MediaInstanceApi.md#delete_media) | **DELETE** /2010-04-01/Accounts/{AccountSid}/Messages/{MessageSid}/Media/{Sid}.json | Delete the Media resource.
[**fetch_media**](Api20100401MediaInstanceApi.md#fetch_media) | **GET** /2010-04-01/Accounts/{AccountSid}/Messages/{MessageSid}/Media/{Sid}.json | Fetch a single Media resource associated with a specific Message resource



## delete_media

> delete_media(account_sid, message_sid, sid)
Delete the Media resource.

Delete the Media resource.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that is associated with the Media resource. | [required] |
**message_sid** | **String** | The SID of the Message resource that is associated with the Media resource. | [required] |
**sid** | **String** | The unique identifier of the to-be-deleted Media resource. | [required] |

### Return type

 (empty response body)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## fetch_media

> models::ApiV2010AccountMessageMedia fetch_media(account_sid, message_sid, sid)
Fetch a single Media resource associated with a specific Message resource

Fetch a single Media resource associated with a specific Message resource

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) associated with the Media resource. | [required] |
**message_sid** | **String** | The SID of the Message resource that is associated with the Media resource. | [required] |
**sid** | **String** | The Twilio-provided string that uniquely identifies the Media resource to fetch. | [required] |

### Return type

[**models::ApiV2010AccountMessageMedia**](api.v2010.account.message.media.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

