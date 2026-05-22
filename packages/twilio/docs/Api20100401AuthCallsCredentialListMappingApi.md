# \Api20100401AuthCallsCredentialListMappingApi

All URIs are relative to *https://api.twilio.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_sip_auth_calls_credential_list_mapping**](Api20100401AuthCallsCredentialListMappingApi.md#create_sip_auth_calls_credential_list_mapping) | **POST** /2010-04-01/Accounts/{AccountSid}/SIP/Domains/{DomainSid}/Auth/Calls/CredentialListMappings.json | Create a new credential list mapping resource
[**delete_sip_auth_calls_credential_list_mapping**](Api20100401AuthCallsCredentialListMappingApi.md#delete_sip_auth_calls_credential_list_mapping) | **DELETE** /2010-04-01/Accounts/{AccountSid}/SIP/Domains/{DomainSid}/Auth/Calls/CredentialListMappings/{Sid}.json | Delete a credential list mapping from the requested domain
[**fetch_sip_auth_calls_credential_list_mapping**](Api20100401AuthCallsCredentialListMappingApi.md#fetch_sip_auth_calls_credential_list_mapping) | **GET** /2010-04-01/Accounts/{AccountSid}/SIP/Domains/{DomainSid}/Auth/Calls/CredentialListMappings/{Sid}.json | Fetch a specific instance of a credential list mapping
[**list_sip_auth_calls_credential_list_mapping**](Api20100401AuthCallsCredentialListMappingApi.md#list_sip_auth_calls_credential_list_mapping) | **GET** /2010-04-01/Accounts/{AccountSid}/SIP/Domains/{DomainSid}/Auth/Calls/CredentialListMappings.json | Retrieve a list of credential list mappings belonging to the domain used in the request



## create_sip_auth_calls_credential_list_mapping

> models::ApiV2010AccountSipSipDomainSipAuthSipAuthCallsSipAuthCallsCredentialListMapping create_sip_auth_calls_credential_list_mapping(account_sid, domain_sid, credential_list_sid)
Create a new credential list mapping resource

Create a new credential list mapping resource

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that will create the resource. | [required] |
**domain_sid** | **String** | The SID of the SIP domain that will contain the new resource. | [required] |
**credential_list_sid** | **String** | The SID of the CredentialList resource to map to the SIP domain. | [required] |

### Return type

[**models::ApiV2010AccountSipSipDomainSipAuthSipAuthCallsSipAuthCallsCredentialListMapping**](api.v2010.account.sip.sip_domain.sip_auth.sip_auth_calls.sip_auth_calls_credential_list_mapping.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: application/x-www-form-urlencoded
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_sip_auth_calls_credential_list_mapping

> delete_sip_auth_calls_credential_list_mapping(account_sid, domain_sid, sid)
Delete a credential list mapping from the requested domain

Delete a credential list mapping from the requested domain

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created the CredentialListMapping resources to delete. | [required] |
**domain_sid** | **String** | The SID of the SIP domain that contains the resource to delete. | [required] |
**sid** | **String** | The Twilio-provided string that uniquely identifies the CredentialListMapping resource to delete. | [required] |

### Return type

 (empty response body)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## fetch_sip_auth_calls_credential_list_mapping

> models::ApiV2010AccountSipSipDomainSipAuthSipAuthCallsSipAuthCallsCredentialListMapping fetch_sip_auth_calls_credential_list_mapping(account_sid, domain_sid, sid)
Fetch a specific instance of a credential list mapping

Fetch a specific instance of a credential list mapping

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created the CredentialListMapping resource to fetch. | [required] |
**domain_sid** | **String** | The SID of the SIP domain that contains the resource to fetch. | [required] |
**sid** | **String** | The Twilio-provided string that uniquely identifies the CredentialListMapping resource to fetch. | [required] |

### Return type

[**models::ApiV2010AccountSipSipDomainSipAuthSipAuthCallsSipAuthCallsCredentialListMapping**](api.v2010.account.sip.sip_domain.sip_auth.sip_auth_calls.sip_auth_calls_credential_list_mapping.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_sip_auth_calls_credential_list_mapping

> models::ListSipAuthCallsCredentialListMappingResponse list_sip_auth_calls_credential_list_mapping(account_sid, domain_sid, page_size, page, page_token)
Retrieve a list of credential list mappings belonging to the domain used in the request

Retrieve a list of credential list mappings belonging to the domain used in the request

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created the CredentialListMapping resources to read. | [required] |
**domain_sid** | **String** | The SID of the SIP domain that contains the resources to read. | [required] |
**page_size** | Option<**i64**> | How many resources to return in each list page. The default is 50, and the maximum is 1000. |  |
**page** | Option<**i32**> | The page index. This value is simply for client state. |  |
**page_token** | Option<**String**> | The page token. This is provided by the API. |  |

### Return type

[**models::ListSipAuthCallsCredentialListMappingResponse**](ListSipAuthCallsCredentialListMappingResponse.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

